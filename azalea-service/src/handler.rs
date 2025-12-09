use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use azalea_log as log;
use tokio::sync::broadcast;

use super::{Service, Status};

pub struct ListenerHandle(
    Arc<broadcast::Sender<()>>,
    Option<tokio::task::JoinHandle<()>>,
);
pub struct LocalListenerHandle(Arc<broadcast::Sender<()>>, Option<glib::JoinHandle<()>>);

impl ListenerHandle {
    pub async fn join(mut self) {
        if let Some(handle) = self.1.take() {
            drop(handle.await);
        }
    }
}

impl LocalListenerHandle {
    pub async fn join(mut self) {
        if let Some(handle) = self.1.take() {
            drop(handle.await);
        }
    }
}

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        let cancellation_sender = &self.0;

        if Arc::strong_count(&cancellation_sender) == 2 {
            drop(cancellation_sender.send(()));
        }

        if let Some(handle) = &self.1 {
            handle.abort();
        }
    }
}

impl Drop for LocalListenerHandle {
    fn drop(&mut self) {
        let cancellation_sender = &self.0;

        if Arc::strong_count(&cancellation_sender) == 2 {
            drop(cancellation_sender.send(()));
        }

        if let Some(handle) = &self.1 {
            handle.abort();
        }
    }
}

/// Service handler responsible for managing/handling a service
#[derive(Clone)]
pub struct Handler<S>
where
    S: Service,
{
    input: flume::Sender<S::Input>,
    output: broadcast::Sender<S::Output>,
    cancellation: Arc<broadcast::Sender<()>>,
    init: S::Init,
    status: Arc<Mutex<Option<flume::Receiver<S::Input>>>>,
}

impl<S> Handler<S>
where
    S: Service + 'static,
{
    pub fn new(init: S::Init, input_capacity: usize, output_capacity: usize) -> Self {
        let (input_sender, input_receiver) = flume::bounded(input_capacity);
        let (output_sender, _) = broadcast::channel(output_capacity);
        let (cancellation_sender, _) = broadcast::channel(1);

        Self {
            input: input_sender,
            output: output_sender,
            init,
            cancellation: Arc::new(cancellation_sender),
            status: Arc::new(Mutex::new(Some(input_receiver))),
        }
    }

    fn _start(&mut self, local: bool) {
        let Some(input) = self.status.lock().unwrap().take() else {
            return;
        };

        let input_sender = self.input.clone();
        let output_sender = self.output.clone();
        let init = self.init.clone();
        let mut cancellation_receiver = self.cancellation.subscribe();
        let status = self.status.clone();

        let task = async move {
            let mut service = S::new(init, input_sender, output_sender.clone()).await;
            let thread_id = std::thread::current().id();
            log::info!(S, "Service started at thread: {:?}", thread_id);

            loop {
                tokio::select! {
                    event = service.event_generator(), if !S::DISABLE_EVENTS => {
                        match service.event_handler(event, &output_sender).await {
                            Ok(_) => continue,
                            Err(e) => log::debug!(S, "Service iteration failed {}", e),
                        }
                    },
                    Ok(msg) = input.recv_async() => service.message(msg, &output_sender).await,
                    _ = cancellation_receiver.recv() => break,
                    else => continue,
                };
            }

            if let Ok(mut status) = status.lock() {
                *status = Some(input);
            };

            log::info!(S, "Service stopped");
        };

        if local {
            relm4::spawn(task);
        } else {
            relm4::spawn_blocking(move || {
                tokio::runtime::Handle::current().block_on(task);
            });
        }
    }

    pub fn start(&mut self) {
        if S::LOCAL {
            self._start(true);
        } else {
            self._start(false);
        }
    }

    pub fn stop(&mut self) {
        if self.status.lock().unwrap().is_none() {
            drop(self.cancellation.send(()));
        }
    }

    pub fn status(&self) -> Status {
        if self.status.lock().unwrap().is_none() {
            Status::Started
        } else {
            Status::Stopped
        }
    }

    pub fn send(&mut self, message: S::Input) {
        drop(self.input.send(message));
    }

    pub fn listen<F: (Fn(S::Output) -> bool) + Send + 'static>(
        &mut self,
        transform: F,
    ) -> ListenerHandle {
        self.start();

        let mut output = self.output.subscribe();

        ListenerHandle(
            self.cancellation.clone(),
            Some(relm4::spawn(async move {
                use tokio::sync::broadcast::error::RecvError;
                loop {
                    if match output.recv().await {
                        Err(RecvError::Closed) => false,
                        Err(RecvError::Lagged(_)) => true,
                        Ok(event) => transform(event),
                    } {
                        continue;
                    } else {
                        break;
                    }
                }
            })),
        )
    }

    pub fn forward<X: Send + 'static, F: (Fn(S::Output) -> X) + Send + 'static>(
        &mut self,
        sender: relm4::Sender<X>,
        transform: F,
    ) -> ListenerHandle {
        self.listen(move |event| sender.send(transform(event)).is_ok())
    }

    pub fn filtered_forward<X: Send + 'static, F: (Fn(S::Output) -> Option<X>) + Send + 'static>(
        &mut self,
        sender: relm4::Sender<X>,
        transform: F,
    ) -> ListenerHandle {
        self.listen(move |event| match transform(event) {
            Some(data) => sender.send(data).is_ok(),
            None => true,
        })
    }

    pub fn listen_local<F: (Fn(S::Output) -> bool) + 'static>(
        &mut self,
        transform: F,
    ) -> LocalListenerHandle {
        self.start();

        let mut output = self.output.subscribe();

        LocalListenerHandle(
            self.cancellation.clone(),
            Some(relm4::spawn_local(async move {
                use tokio::sync::broadcast::error::RecvError;
                loop {
                    if match output.recv().await {
                        Err(RecvError::Closed) => false,
                        Err(RecvError::Lagged(_)) => true,
                        Ok(event) => transform(event),
                    } {
                        continue;
                    } else {
                        break;
                    }
                }
            })),
        )
    }

    pub fn forward_local<X: 'static, F: (Fn(S::Output) -> X) + 'static>(
        &mut self,
        sender: relm4::Sender<X>,
        transform: F,
    ) -> LocalListenerHandle {
        self.listen_local(move |event| sender.send(transform(event)).is_ok())
    }

    pub fn filtered_forward_local<X: 'static, F: (Fn(S::Output) -> Option<X>) + 'static>(
        &mut self,
        sender: relm4::Sender<X>,
        transform: F,
    ) -> LocalListenerHandle {
        self.listen_local(move |event| match transform(event) {
            Some(data) => sender.send(data).is_ok(),
            None => true,
        })
    }
}

/// Thread local static handler for a Service
///
/// This allows global (thread local) access to a service
pub trait LocalStaticHandler
where
    Self: Service,
{
    fn static_handler() -> Rc<RefCell<crate::Handler<Self>>>;

    fn init(init: Self::Init) {
        Self::static_handler().borrow_mut().init = init
    }

    fn start() {
        Self::static_handler().borrow_mut().start()
    }

    fn stop() {
        Self::static_handler().borrow_mut().stop()
    }

    fn status() -> crate::Status {
        Self::static_handler().borrow().status()
    }

    fn send(message: Self::Input) {
        Self::static_handler().borrow_mut().send(message)
    }

    fn listen<F: (Fn(Self::Output) -> bool) + Send + 'static>(
        transform: F,
    ) -> crate::ListenerHandle {
        Self::static_handler().borrow_mut().listen(transform)
    }

    fn forward<X: Send + 'static, F: (Fn(Self::Output) -> X) + Send + 'static>(
        sender: relm4::Sender<X>,
        transform: F,
    ) -> crate::ListenerHandle {
        Self::static_handler()
            .borrow_mut()
            .forward(sender, transform)
    }

    fn filtered_forward<X: Send + 'static, F: (Fn(Self::Output) -> Option<X>) + Send + 'static>(
        sender: relm4::Sender<X>,
        transform: F,
    ) -> crate::ListenerHandle {
        Self::static_handler()
            .borrow_mut()
            .filtered_forward(sender, transform)
    }

    fn listen_local<F: (Fn(Self::Output) -> bool) + 'static>(
        transform: F,
    ) -> crate::LocalListenerHandle {
        Self::static_handler().borrow_mut().listen_local(transform)
    }

    fn forward_local<X: 'static, F: (Fn(Self::Output) -> X) + 'static>(
        sender: relm4::Sender<X>,
        transform: F,
    ) -> crate::LocalListenerHandle {
        Self::static_handler()
            .borrow_mut()
            .forward_local(sender, transform)
    }

    fn filtered_forward_local<X: 'static, F: (Fn(Self::Output) -> Option<X>) + 'static>(
        sender: relm4::Sender<X>,
        transform: F,
    ) -> crate::LocalListenerHandle {
        Self::static_handler()
            .borrow_mut()
            .filtered_forward_local(sender, transform)
    }
}

/// Static handler for a Service
///
/// This allows global (multi thread) access to a service
pub trait StaticHandler
where
    Self: Service,
{
    fn static_handler() -> Arc<Mutex<crate::Handler<Self>>>;

    fn init(init: Self::Init) {
        match Self::static_handler().lock() {
            Ok(mut handler) => handler.init = init,
            Err(e) => azalea_log::warning!(Self, "Failed to lock service handler: {}", e),
        }
    }

    fn start() {
        match Self::static_handler().lock() {
            Ok(mut handler) => handler.start(),
            Err(e) => azalea_log::warning!(Self, "Failed to lock service handler: {}", e),
        }
    }

    fn stop() {
        match Self::static_handler().lock() {
            Ok(mut handler) => handler.stop(),
            Err(e) => azalea_log::warning!(Self, "Failed to lock service handler: {}", e),
        }
    }

    fn status() -> crate::Status {
        match Self::static_handler().lock() {
            Ok(handler) => handler.status(),
            Err(e) => {
                azalea_log::warning!(Self, "Failed to lock service handler: {}", e);
                crate::Status::Stopped
            }
        }
    }

    fn send(message: Self::Input) {
        match Self::static_handler().lock() {
            Ok(mut handler) => handler.send(message),
            Err(e) => azalea_log::warning!(Self, "Failed to lock service handler: {}", e),
        }
    }

    fn listen<F: (Fn(Self::Output) -> bool) + Send + 'static>(
        transform: F,
    ) -> crate::ListenerHandle {
        match Self::static_handler().lock() {
            Ok(mut handler) => handler.listen(transform),
            Err(e) => azalea_log::error!(Self, "Failed to lock service handler: {}", e),
        }
    }

    fn forward<X: Send + 'static, F: (Fn(Self::Output) -> X) + Send + 'static>(
        sender: relm4::Sender<X>,
        transform: F,
    ) -> crate::ListenerHandle {
        match Self::static_handler().lock() {
            Ok(mut handler) => handler.forward(sender, transform),
            Err(e) => azalea_log::error!(Self, "Failed to lock service handler: {}", e),
        }
    }

    fn filtered_forward<X: Send + 'static, F: (Fn(Self::Output) -> Option<X>) + Send + 'static>(
        sender: relm4::Sender<X>,
        transform: F,
    ) -> crate::ListenerHandle {
        match Self::static_handler().lock() {
            Ok(mut handler) => handler.filtered_forward(sender, transform),
            Err(e) => azalea_log::error!(Self, "Failed to lock service handler: {}", e),
        }
    }

    fn listen_local<F: (Fn(Self::Output) -> bool) + 'static>(
        transform: F,
    ) -> crate::LocalListenerHandle {
        match Self::static_handler().lock() {
            Ok(mut handler) => handler.listen_local(transform),
            Err(e) => azalea_log::error!(Self, "Failed to lock service handler: {}", e),
        }
    }

    fn forward_local<X: 'static, F: (Fn(Self::Output) -> X) + 'static>(
        sender: relm4::Sender<X>,
        transform: F,
    ) -> crate::LocalListenerHandle {
        match Self::static_handler().lock() {
            Ok(mut handler) => handler.forward_local(sender, transform),
            Err(e) => azalea_log::error!(Self, "Failed to lock service handler: {}", e),
        }
    }

    fn filtered_forward_local<X: 'static, F: (Fn(Self::Output) -> Option<X>) + 'static>(
        sender: relm4::Sender<X>,
        transform: F,
    ) -> crate::LocalListenerHandle {
        match Self::static_handler().lock() {
            Ok(mut handler) => handler.filtered_forward_local(sender, transform),
            Err(e) => azalea_log::error!(Self, "Failed to lock service handler: {}", e),
        }
    }
}

#[macro_export]
macro_rules! impl_local_static_handler {
    ($service:ty) => {
        impl $crate::LocalStaticHandler for $service {
            fn static_handler() -> std::rc::Rc<std::cell::RefCell<$crate::Handler<Self>>> {
                use std::{cell::RefCell, rc::Rc, sync::LazyLock};

                thread_local! {
                    static HANDLER: LazyLock<Rc<RefCell<$crate::Handler<$service>>>> = LazyLock::new(|| {
                        azalea_log::debug!($service, "Service initialized");
                        Rc::new(RefCell::new(<$service as $crate::Service>::handler(
                            Default::default(),
                        )))
                    });
                }

                HANDLER.with(|handler| {
                    (*handler).clone()
                })
            }
        }
    }
}

#[macro_export]
macro_rules! impl_static_handler {
    ($service:ty) => {
        impl $crate::StaticHandler for Service {
            fn static_handler() -> std::sync::Arc<std::sync::Mutex<$crate::Handler<Self>>> {
                use std::sync::{Arc, LazyLock, Mutex};

                static HANDLER: LazyLock<Arc<Mutex<$crate::Handler<$service>>>> =
                    LazyLock::new(|| {
                        azalea_log::debug!($service, "Service initialized");

                        Arc::new(Mutex::new(<$service as $crate::Service>::handler(
                            Default::default(),
                        )))
                    });

                HANDLER.clone()
            }
        }
    };
}
