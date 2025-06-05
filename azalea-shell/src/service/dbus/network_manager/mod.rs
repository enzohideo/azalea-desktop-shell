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
    connectivity: PropertyStream<'static, NMConnectivityState>,
}

#[derive(Default, Clone)]
pub struct Init {
    pub dbus_connection: Option<zbus::Connection>,
}

#[derive(Clone, Debug)]
pub enum Output {
    StateChanged(NMState),
    ConnectivityChanged(NMConnectivityState),
}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = ();
    type Event = Output;
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

        azalea_log::debug::<Self>(&format!(
            "Version: {}",
            proxy.version().await.unwrap_or_default()
        ));

        Self {
            streams: Streams {
                state: proxy.receive_state_changed().await,
                connectivity: proxy.receive_connectivity_changed().await,
            },
            proxy,
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
            tokio::select! {
                Some(prop) = self.streams.state.next() => {
                    let Ok(value) = prop.get().await else {
                        continue;
                    };
                    return Output::StateChanged(value);
                },
                Some(prop) = self.streams.connectivity.next() => {
                    let Ok(value) = prop.get().await else {
                        continue;
                    };
                    return Output::ConnectivityChanged(value);
                },
                else => continue,
            }
        }
    }

    async fn event_handler(
        &mut self,
        event: Self::Event,
        output_sender: &tokio::sync::broadcast::Sender<Self::Output>,
    ) -> Result<(), error::Error> {
        output_sender.send(event)?;
        Ok(())
    }
}
