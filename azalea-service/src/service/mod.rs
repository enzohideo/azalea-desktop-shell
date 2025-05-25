mod handler;
pub use handler::*;
mod status;
pub use status::*;
pub mod error;

use tokio::sync::broadcast;

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
    ) -> impl std::future::Future<Output = Result<(), error::Error>> + Send;
}
