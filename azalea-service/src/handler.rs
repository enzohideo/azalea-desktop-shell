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

    pub fn start(&mut self) {
        let Some(input) = self.status.lock().unwrap().take() else {
            return;
        };

        let input_sender = self.input.clone();
        let output_sender = self.output.clone();
        let init = self.init.clone();
        let mut cancellation_receiver = self.cancellation.subscribe();
        let status = self.status.clone();

        relm4::spawn(async move {
            let mut service = S::new(init, input_sender, output_sender.clone()).await;
            log::info::<S>("Service started");

            loop {
                tokio::select! {
                    event = service.event_generator(), if !S::DISABLE_EVENTS => {
                        match service.event_handler(event, &output_sender).await {
                            Ok(_) => continue,
                            Err(e) => azalea_log::debug::<S>(&format!("Service iteration failed {e}")),
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

            log::info::<S>("Service stopped");
        });
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

pub trait StaticHandler
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

#[macro_export]
macro_rules! impl_static_handler {
    ($service:ty) => {
        impl $crate::StaticHandler for $service {
            fn static_handler() -> std::rc::Rc<std::cell::RefCell<$crate::Handler<Self>>> {
                use std::{cell::RefCell, rc::Rc, sync::OnceLock};

                thread_local! {
                    static HANDLER: OnceLock<Rc<RefCell<$crate::Handler<$service>>>> = OnceLock::new();
                }

                HANDLER.with(|handler| {
                    handler
                        .get_or_init(move || {
                            azalea_log::debug::<$service>("Service initialized");
                            Rc::new(RefCell::new(<Self as $crate::Service>::handler(
                                Default::default(),
                            )))
                        })
                        .clone()
                })
            }
        }
    }
}
