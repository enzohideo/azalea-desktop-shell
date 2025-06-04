pub mod proxy;

use proxy::{NMConnectivityState, NMState, NetworkManagerProxy};
use tokio::sync::broadcast;

use azalea_service::error;

#[derive(azalea_derive::StaticHandler)]
pub struct Service {
    proxy: NetworkManagerProxy<'static>,
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
    type Event = ();
    type Output = Output;
    const DISABLE_EVENTS: bool = true;

    async fn new(
        init: Self::Init,
        _: flume::Sender<Self::Input>,
        _: broadcast::Sender<Self::Output>,
    ) -> Self {
        let connection = init
            .dbus_connection
            .unwrap_or(zbus::Connection::system().await.unwrap());
        let proxy = NetworkManagerProxy::new(&connection).await.unwrap();

        println!("{:?}", proxy.state().await);
        println!("{:?}", proxy.connectivity().await);

        Self { proxy }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) {
        println!("Received input {input:?}");
    }
}
