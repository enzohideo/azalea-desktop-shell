use tokio::sync::broadcast;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Service handled with success")]
    Success,
    #[error("Service failed to handle")]
    Failure,
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
