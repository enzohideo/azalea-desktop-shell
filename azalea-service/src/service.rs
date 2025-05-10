use relm4::WorkerController;

#[derive(Debug)]
pub enum Input {
    Start,
    Stop,
}

pub struct Service<Model>
where
    Model: relm4::Worker,
{
    #[allow(dead_code)]
    worker: WorkerController<Model>,
    sender: tokio::sync::broadcast::Sender<Model::Output>,
}

impl<Model> Service<Model>
where
    Model: relm4::Worker<Input = Input>
        + relm4::Component<Input = Input, Root = (), Widgets = ()>
        + Send,
    <Model as relm4::Component>::Output: Send,
    <Model as relm4::Worker>::Output: Clone,
    <Model as relm4::Component>::CommandOutput: Send,
{
    pub fn new(init: <Model as relm4::Component>::Init) -> Self {
        <Model as relm4::Component>::builder()
            .detach_worker(init)
            .into()
    }

    pub fn send(&self, message: Input) -> Result<(), Input> {
        self.worker.sender().send(message)
    }

    pub fn forward<X: 'static, F: (Fn(<Model as relm4::Worker>::Output) -> X) + 'static>(
        &self,
        sender: relm4::Sender<X>,
        transform: F,
    ) {
        let mut receiver = self.sender.subscribe();
        relm4::spawn_local(async move {
            while let Ok(event) = receiver.recv().await {
                if sender.send(transform(event)).is_err() {
                    return;
                }
            }
        });
    }
}

impl<Model> From<relm4::WorkerHandle<Model>> for Service<Model>
where
    Model: relm4::Worker,
    <Model as relm4::Worker>::Output: Clone,
{
    fn from(value: relm4::WorkerHandle<Model>) -> Self {
        let (flume_sender, flume_receiver) = flume::unbounded();

        let flume_sender: relm4::Sender<Model::Output> = flume_sender.into();
        let worker = value.forward(&flume_sender, |v| v);

        let (tokio_sender, _) = tokio::sync::broadcast::channel(1);
        let sender = tokio_sender.clone();

        relm4::spawn_local(async move {
            while let Ok(event) = flume_receiver.recv_async().await {
                if tokio_sender.send(event).is_err() {
                    return;
                }
            }
        });

        Self { worker, sender }
    }
}
