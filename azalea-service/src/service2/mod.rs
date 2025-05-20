use tokio::sync::{broadcast, mpsc};

pub enum Status<S>
where
    S: Service,
{
    Started(mpsc::Sender<S::Input>, relm4::JoinHandle<()>),
    Stopped,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Service handled with success")]
    Success,
    #[error("Service failed to handle")]
    Failure
}

impl<T> From<broadcast::error::SendError<T>> for Error {
    fn from(_value: broadcast::error::SendError<T>) -> Self {
        Self::Failure
    }
}

impl From<broadcast::error::RecvError> for Error {
    fn from(_value: broadcast::error::RecvError) -> Self {
        Self::Failure
    }
}

pub trait Service
where
    Self: Sized + Send + 'static,
{
    type Init: Clone + Send;
    type Input: Clone + Send;
    type Output: Clone + 'static + Send;

    fn new(init: Self::Init) -> Self;
    fn handler(init: Self::Init) -> Handler<Self> {
        Handler::new(init)
    }
    fn message(&self, input: Self::Input, output: &broadcast::Sender<Self::Output>);
    fn iteration(
        &mut self,
        output: &broadcast::Sender<Self::Output>,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
}

pub struct Handler<S>
where
    S: Service,
{
    output: broadcast::Sender<S::Output>,
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
    }

    pub fn stop(&mut self) {
        if let Status::Started(_, join_handler) = &self.status {
            join_handler.abort();
            self.status = Status::Stopped;
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

    pub fn listen<F: (Fn(S::Output) -> bool) + 'static>(&self, transform: F) {
        let mut output_receiver = self.output.subscribe();
        relm4::spawn_local(async move {
            use tokio::sync::broadcast::error::RecvError;
            loop {
                match output_receiver.recv().await {
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                    Ok(event) => {
                        if !transform(event) {
                            break;
                        }
                    }
                };
            }
        });
    }

    pub fn forward<X: 'static, F: (Fn(S::Output) -> X) + 'static>(
        &self,
        sender: relm4::Sender<X>,
        transform: F,
    ) {
        self.listen(move |event| sender.send(transform(event)).is_err())
    }
}
