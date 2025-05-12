use std::{marker::PhantomData, sync::mpsc};

use relm4::ComponentSender;

use crate::Input;

pub mod time;

pub trait WorkerExt<W>: Sized + Send + 'static
where
    W: relm4::Worker,
{
    type Init: 'static + Send;
    type Output: 'static + Send + std::fmt::Debug;

    fn new(init: Self::Init) -> Self;
    fn update(
        &mut self,
        sender: &ComponentSender<W>,
    ) -> impl std::future::Future<Output = ()> + Send
    where
        W: relm4::Worker;
}

pub struct Model<W>
where
    W: WorkerExt<Self>,
{
    sender: mpsc::Sender<Input>,
    phantom: PhantomData<W>,
}

impl<W> relm4::Worker for Model<W>
where
    W: WorkerExt<Self>,
{
    type Init = W::Init;
    type Input = crate::Input;
    type Output = W::Output;

    fn init(init: Self::Init, sender: ComponentSender<Self>) -> Self {
        let (tx, rx) = mpsc::channel();
        let sender_clone = sender.clone();

        sender.oneshot_command(async move {
            let mut state = W::new(init);

            while let Ok(input) = rx.recv() {
                match input {
                    Input::Start => loop {
                        if let Ok(Input::Stop) = rx.try_recv() {
                            break;
                        }
                        state.update(&sender_clone).await;
                    },
                    Input::Stop => continue,
                }
            }
        });

        Self {
            sender: tx,
            phantom: Default::default(),
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        drop(self.sender.send(message));
    }
}
