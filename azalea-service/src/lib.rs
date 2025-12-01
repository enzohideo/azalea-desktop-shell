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
    type Input: Send;
    type Event: Send;
    type Output: Clone + 'static + Send;
    const DISABLE_EVENTS: bool = false;

    fn new(
        init: Self::Init,
        input_sender: flume::Sender<Self::Input>,
        output_sender: broadcast::Sender<Self::Output>,
    ) -> impl std::future::Future<Output = Self> + Send;

    fn handler(init: Self::Init) -> Handler<Self> {
        Handler::new(init, 1, 1)
    }

    fn message(
        &mut self,
        _input: Self::Input,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) -> impl std::future::Future<Output = ()> + Send {
        async {}
    }

    fn event_generator(&mut self) -> impl std::future::Future<Output = Self::Event> + Send {
        async {
            azalea_log::error!(Self, "Event generator not implemented!");
            panic!()
        }
    }

    fn event_handler(
        &mut self,
        _event: Self::Event,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
        async { Ok(()) }
    }
}

pub use anyhow::Result;
