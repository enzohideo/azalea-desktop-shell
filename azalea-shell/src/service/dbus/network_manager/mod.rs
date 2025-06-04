pub mod proxy;

use futures_lite::StreamExt;
use proxy::{NMConnectivityState, NMState, NetworkManagerProxy};
use tokio::sync::broadcast;

use azalea_service::error;
use zbus::proxy::PropertyStream;

#[derive(azalea_derive::StaticHandler)]
pub struct Service {
    proxy: NetworkManagerProxy<'static>,
    streams: Streams,
}

pub struct Streams {
    state: PropertyStream<'static, NMState>,
}

#[derive(Default, Clone)]
pub struct Init {
    pub dbus_connection: Option<zbus::Connection>,
}

#[derive(Debug)]
pub enum Event {
    State(NMState),
}

#[derive(Clone, Debug)]
pub enum Output {
    StateChanged(NMState),
    ConnectivityChanged(NMConnectivityState),
}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = ();
    type Event = Event;
    type Output = Output;

    async fn new(
        init: Self::Init,
        _: flume::Sender<Self::Input>,
        _: broadcast::Sender<Self::Output>,
    ) -> Self {
        let connection = init
            .dbus_connection
            .unwrap_or(zbus::Connection::system().await.unwrap());
        let proxy = NetworkManagerProxy::new(&connection).await.unwrap();

        let listener = proxy.receive_state_changed();
        let state_stream = listener.await;

        Self {
            proxy,
            streams: Streams {
                state: state_stream,
            },
        }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) {
        println!("Received input {input:?}");
    }

    async fn event_generator(&mut self) -> Self::Event {
        loop {
            let prop = self.streams.state.next().await.unwrap();
            let Ok(state) = prop.get().await else {
                continue;
            };
            return Event::State(state);
        }
    }

    async fn event_handler(
        &mut self,
        event: Self::Event,
        _output_sender: &tokio::sync::broadcast::Sender<Self::Output>,
    ) -> Result<(), error::Error> {
        println!("{:?}", event);
        Ok(())
    }
}
