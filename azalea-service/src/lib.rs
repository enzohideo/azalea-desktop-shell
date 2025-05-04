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
    Model: relm4::Worker<Input = ()> + relm4::Component<Root = (), Widgets = ()> + Send,
    <Model as relm4::Component>::Input: Send,
    <Model as relm4::Component>::Output: Send,
    <Model as relm4::Component>::CommandOutput: Send,
{
    pub fn new(init: <Model as relm4::Component>::Init) -> Self {
        <Model as relm4::Component>::builder()
            .detach_worker(init)
            .into()
    }

    pub fn forward<X: 'static, F: (Fn(<Model as relm4::Worker>::Output) -> X) + 'static>(
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
