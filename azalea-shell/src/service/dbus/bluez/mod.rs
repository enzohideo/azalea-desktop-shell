use std::collections::HashSet;

use tokio::sync::broadcast;

use zbus::fdo::ObjectManagerProxy;

#[derive(azalea_derive::StaticHandler)]
pub struct Service {
    interface: BluetoothInterface,
}

pub struct BluetoothInterface {
    selected: String,
    interfaces: HashSet<String>,
}

impl BluetoothInterface {
    async fn new<'a>(proxy: &ObjectManagerProxy<'a>) -> Self {
        let interfaces = HashSet::<String>::from_iter(
            proxy
                .get_managed_objects()
                .await
                .unwrap()
                .keys()
                .flat_map(|p| p.split("/").nth(3).map(|s| s.to_owned())),
        );

        Self {
            selected: interfaces
                .iter()
                .nth(0)
                .map(|v| v.to_owned())
                .unwrap_or(format!("hci0")),
            interfaces,
        }
    }
}

#[derive(Default, Clone)]
pub struct Init {
    pub dbus_connection: Option<zbus::Connection>,
}

#[derive(Clone, Debug)]
pub enum Input {}

pub enum Event {}

#[derive(Clone, Debug)]
pub enum Output {}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = Input;
    type Event = ();
    type Output = Output;

    async fn new(
        init: Self::Init,
        _input: flume::Sender<Self::Input>,
        _: broadcast::Sender<Self::Output>,
    ) -> Self {
        let connection = init
            .dbus_connection
            .unwrap_or(zbus::Connection::system().await.unwrap());
        let proxy = zbus::fdo::ObjectManagerProxy::new(&connection, "org.bluez", "/")
            .await
            .unwrap();

        let interface = BluetoothInterface::new(&proxy).await;

        Self { interface }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        output_sender: &broadcast::Sender<Self::Output>,
    ) {
    }

    async fn event_generator(&mut self) -> Self::Event {}

    async fn event_handler(
        &mut self,
        event: Self::Event,
        output_sender: &tokio::sync::broadcast::Sender<Self::Output>,
    ) -> azalea_service::Result<()> {
        Ok(())
    }
}
