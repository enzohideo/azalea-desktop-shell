use std::sync::mpsc;

use relm4::{ComponentSender, Worker};

use crate::Input;

pub type Output = chrono::DateTime<chrono::Local>;

#[derive(Debug)]
pub struct Model {
    sender: mpsc::Sender<Input>,
}

// Add WorkerExt and impl Worker for WorkerExt
impl Worker for Model {
    type Init = std::time::Duration;
    type Input = Input;
    type Output = Output;

    fn init(init: Self::Init, sender: ComponentSender<Self>) -> Self {
        let (tx, rx) = mpsc::channel();
        let output = sender.output_sender().clone();

        sender.oneshot_command(async move {
            while let Ok(input) = rx.recv() {
                match input {
                    Input::Start => loop {
                        if let Ok(Input::Stop) = rx.try_recv() {
                            break;
                        }
                        let _ = output.send(chrono::Local::now());
                        tokio::time::sleep(init).await;
                    },
                    Input::Stop => continue,
                }
            }
        });

        Self { sender: tx }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>) {
        drop(self.sender.send(input));
    }
}
