use std::sync::{Arc, Mutex};

use azalea_log as log;
use tokio::sync::broadcast;

use super::{Service, Status};

pub struct ListenerHandle(Arc<broadcast::Sender<()>>, gtk::glib::JoinHandle<()>);

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        let cancellation_sender = &self.0;

        if Arc::strong_count(&cancellation_sender) == 2 {
            drop(cancellation_sender.send(()));
        }

        self.1.abort();
    }
}

pub struct Handler<S>
where
    S: Service,
{
    input: broadcast::Sender<S::Input>,
    output: broadcast::Sender<S::Output>,
    cancellation: Arc<broadcast::Sender<()>>,
    init: S::Init,
    status: Arc<Mutex<Status>>,
}

impl<S> Handler<S>
where
    S: Service + 'static,
{
    pub fn new(init: S::Init) -> Self {
        // TODO: Receive channel limits (somehow)
        let (input_sender, _) = broadcast::channel(10);
        let (output_sender, _) = broadcast::channel(10);
        let (cancellation_sender, _) = broadcast::channel(1);

        Self {
            input: input_sender,
            output: output_sender,
            init,
            cancellation: Arc::new(cancellation_sender),
            status: Arc::new(Mutex::new(Status::Stopped)),
        }
    }

    pub fn start(&mut self) {
        self.stop();
        let input_sender = self.input.clone();
        let mut input = self.input.subscribe();
        let output_sender = self.output.clone();
        let init = self.init.clone();
        // TODO: Receive number of channels
        let mut cancellation_receiver = self.cancellation.subscribe();
        let status = self.status.clone();

        relm4::spawn(async move {
            let mut service = S::new(init, input_sender, output_sender.clone()).await;

            loop {
                tokio::select! {
                    _ = service.iteration(&output_sender) => (),
                    Ok(msg) = input.recv() => service.message(msg, &output_sender).await,
                    _ = cancellation_receiver.recv() => break,
                    else => continue,
                };
            }

            if let Ok(mut status) = status.lock() {
                *status = Status::Stopped;
            };

            log::message!("Service was stopped: {}", std::any::type_name::<S>());
        });

        if let Ok(mut status) = self.status.lock() {
            *status = Status::Started;
        };

        log::message!("Service started: {}", std::any::type_name::<S>());
    }

    pub fn stop(&mut self) {
        if let Status::Started = *(self.status.lock().unwrap()) {
            drop(self.cancellation.send(()));
        }
    }

    pub fn status(&self) -> Status {
        (*self.status.lock().unwrap()).clone()
    }

    pub fn send(&mut self, message: S::Input) {
        drop(self.input.send(message));
    }

    pub fn listen<F: (Fn(S::Output) -> bool) + 'static>(&mut self, transform: F) -> ListenerHandle {
        let mut output = self.output.subscribe();

        if Arc::strong_count(&self.cancellation) == 1 {
            self.start();
        }

        ListenerHandle(
            self.cancellation.clone(),
            relm4::spawn_local(async move {
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
            }),
        )
    }

    pub fn forward<X: 'static, F: (Fn(S::Output) -> X) + 'static>(
        &mut self,
        sender: relm4::Sender<X>,
        transform: F,
    ) -> ListenerHandle {
        self.listen(move |event| sender.send(transform(event)).is_ok())
    }

    pub fn filtered_forward<X: 'static, F: (Fn(S::Output) -> Option<X>) + 'static>(
        &mut self,
        sender: relm4::Sender<X>,
        transform: F,
    ) -> ListenerHandle {
        self.listen(move |event| match transform(event) {
            Some(data) => sender.send(data).is_ok(),
            None => true,
        })
    }
}
