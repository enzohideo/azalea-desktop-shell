use relm4::WorkerController;

pub mod time;

pub struct Service<Model>
where
    Model: relm4::Worker,
{
    #[allow(dead_code)]
    worker: WorkerController<Model>,
    receiver: flume::Receiver<Model::Output>,
}

impl<Model> Service<Model>
where
    Model: relm4::Worker,
{
    pub fn forward<X: 'static, F: (Fn(Model::Output) -> X) + 'static>(
        &self,
        sender: relm4::Sender<X>,
        transform: F,
    ) {
        let receiver = self.receiver.clone();
        relm4::spawn_local(async move {
            while let Ok(event) = receiver.recv_async().await {
                if sender.send(transform(event)).is_err() {
                    return;
                }
            }
        });
    }
}

impl<Model> From<relm4::WorkerHandle<Model>> for Service<Model>
where
    Model: relm4::Worker<Input = ()>,
{
    fn from(value: relm4::WorkerHandle<Model>) -> Self {
        let (sender, receiver) = flume::unbounded();

        let sender: relm4::Sender<Model::Output> = sender.into();
        let worker = value.forward(&sender, |v| v);

        let _ = worker.sender().send(());

        Self { worker, receiver }
    }
}
