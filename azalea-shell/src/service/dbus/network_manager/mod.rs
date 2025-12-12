pub mod proxy;

use futures_lite::StreamExt;
use proxy::{NMConnectivityState, NMState, NetworkManagerProxy};
use tokio::sync::broadcast;

use zbus::{
    proxy::{PropertyChanged, PropertyStream},
    zvariant::OwnedObjectPath,
};

#[derive(azalea_derive::StaticHandler)]
pub struct Service {
    #[allow(dead_code)]
    proxy: NetworkManagerProxy<'static>,
    streams: Streams,
}

pub struct Streams {
    enable: PropertyStream<'static, bool>,
    state: PropertyStream<'static, NMState>,
    connectivity: PropertyStream<'static, NMConnectivityState>,
}

#[derive(Default, Clone)]
pub struct Init {
    pub dbus_connection: Option<zbus::Connection>,
}

#[derive(Clone, Debug)]
pub enum Input {
    GetDevices,
    Update,
    Enable(bool),
    ActivateConnection {
        connection: Option<OwnedObjectPath>,
        device: OwnedObjectPath,
        specific_object: Option<OwnedObjectPath>,
    },
}

pub enum Event {
    NetworkingEnabledChanged(PropertyChanged<'static, bool>),
    StateChanged(PropertyChanged<'static, NMState>),
    ConnectivityChanged(PropertyChanged<'static, NMConnectivityState>),
}

#[derive(Clone, Debug)]
pub enum Output {
    Devices(Vec<OwnedObjectPath>),
    NetworkingEnabledChanged(bool),
    StateChanged(NMState),
    ConnectivityChanged(NMConnectivityState),
}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = Input;
    type Event = Event;
    type Output = Output;

    fn handler(init: Self::Init) -> azalea_service::Handler<Self> {
        azalea_service::Handler::new(init, 4, 8)
    }

    async fn new(
        init: Self::Init,
        _input: flume::Sender<Self::Input>,
        _: broadcast::Sender<Self::Output>,
    ) -> Self {
        let connection = init
            .dbus_connection
            .unwrap_or(zbus::Connection::system().await.unwrap());
        let proxy = NetworkManagerProxy::new(&connection).await.unwrap();

        azalea_log::debug!(
            Self,
            "Version: {}",
            proxy.version().await.unwrap_or_default()
        );

        Self {
            streams: Streams {
                enable: proxy.receive_networking_enabled_changed().await,
                state: proxy.receive_state_changed().await,
                connectivity: proxy.receive_connectivity_changed().await,
            },
            proxy,
        }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::GetDevices => {
                drop(output_sender.send(Output::Devices(
                    self.proxy.get_devices().await.unwrap_or_default(),
                )));
            }
            Input::Update => {
                drop(output_sender.send(Output::StateChanged(self.proxy.state().await.unwrap())));
                drop(output_sender.send(Output::ConnectivityChanged(
                    self.proxy.connectivity().await.unwrap(),
                )));
                drop(output_sender.send(Output::Devices(
                    self.proxy.get_devices().await.unwrap_or_default(),
                )));
            }
            Input::Enable(on) => {
                if let Err(e) = self.proxy.enable(on).await {
                    azalea_log::warning!("Failed to (dis)enable network: {}", e)
                }
            }
            Input::ActivateConnection {
                connection,
                device,
                specific_object,
            } => {
                let root_object_path = OwnedObjectPath::try_from("/").unwrap();

                drop(
                    self.proxy
                        .activate_connection(
                            connection.unwrap_or(root_object_path.clone()),
                            device,
                            specific_object.unwrap_or(root_object_path),
                        )
                        .await,
                );
            }
        }
    }

    async fn event_generator(&mut self) -> Self::Event {
        loop {
            tokio::select! {
                Some(prop) = self.streams.enable.next() =>
                    return Event::NetworkingEnabledChanged(prop),
                Some(prop) = self.streams.state.next() =>
                    return Event::StateChanged(prop),
                Some(prop) = self.streams.connectivity.next() =>
                    return Event::ConnectivityChanged(prop),
                else => continue,
            }
        }
    }

    async fn event_handler(
        &mut self,
        event: Self::Event,
        output_sender: &tokio::sync::broadcast::Sender<Self::Output>,
    ) -> azalea_service::Result<()> {
        let output = match event {
            Event::StateChanged(prop) => Output::StateChanged(prop.get().await?),
            Event::ConnectivityChanged(prop) => Output::ConnectivityChanged(prop.get().await?),
            Event::NetworkingEnabledChanged(prop) => {
                Output::NetworkingEnabledChanged(prop.get().await?)
            }
        };
        output_sender.send(output)?;
        Ok(())
    }
}
