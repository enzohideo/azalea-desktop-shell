use std::rc::Rc;

use azalea_core::log;
use tokio::sync::broadcast;

use super::{Service, Status};

// TODO: Use channel to send "pause/stop" request when all handles are dropped
pub struct ListenerHandle(Rc<()>, gtk::glib::JoinHandle<()>);

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        self.1.abort();
    }
}

pub struct Handler<S>
where
    S: Service,
{
    output: broadcast::Sender<S::Output>,
    listeners: Rc<()>,
    init: S::Init,
    status: Status<S>,
}

impl<S> Handler<S>
where
    S: Service + 'static,
{
    pub fn new(init: S::Init) -> Self {
        let (output_sender, _) = broadcast::channel(1);

        Self {
            output: output_sender,
            init,
            listeners: Rc::new(()),
            status: Status::Stopped,
        }
    }

    pub fn start(&mut self) {
        self.stop();
        let output = self.output.clone();
        let init = self.init.clone();
        // TODO: Receive number of channels
        let (input_sender, mut input_receiver) = tokio::sync::mpsc::channel(1);
        let join_handler = relm4::spawn(async move {
            let mut service = S::new(init);
            loop {
                tokio::select! {
                    _ = service.iteration(&output) => (),
                    Some(msg) = input_receiver.recv() => service.message(msg, &output),
                    else => continue,
                };
            }
        });
        self.status = Status::Started(input_sender, join_handler);
        log::message!("Service has started: {}", std::any::type_name::<S>());
    }

    pub fn stop(&mut self) {
        if let Status::Started(_, join_handler) = &self.status {
            join_handler.abort();
            self.status = Status::Stopped;
            log::message!("Service was stopped: {}", std::any::type_name::<S>());
        }
    }

    pub fn status(&self) -> &Status<S> {
        &self.status
    }

    pub fn send(&mut self, message: S::Input) {
        let Status::Started(input, _) = &self.status else {
            return;
        };
        let tx = input.clone();
        relm4::spawn(async move {
            drop(tx.send(message).await);
        });
    }

    pub fn listen<F: (Fn(S::Output) -> bool) + 'static>(&mut self, transform: F) -> ListenerHandle {
        let mut output_receiver = self.output.subscribe();

        if Rc::strong_count(&self.listeners) == 1 {
            self.start();
        }

        ListenerHandle(
            self.listeners.clone(),
            relm4::spawn_local(async move {
                use tokio::sync::broadcast::error::RecvError;
                loop {
                    if match output_receiver.recv().await {
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
