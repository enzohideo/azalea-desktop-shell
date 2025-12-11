pub mod proxy;

use proxy::LoginManagerProxy;
use tokio::sync::broadcast;

#[derive(azalea_derive::StaticHandler)]
pub struct Service {
    #[allow(dead_code)]
    proxy: LoginManagerProxy<'static>,
}

#[derive(Default, Clone)]
pub struct Init {
    pub dbus_connection: Option<zbus::Connection>,
}

#[derive(Clone, Debug)]
pub enum Input {
    PowerOff,
    Reboot,
    Suspend,
    Hibernate,
}

pub enum Event {}

#[derive(Clone, Debug)]
pub enum Output {}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = Input;
    type Event = Event;
    type Output = Output;

    const DISABLE_EVENTS: bool = true;

    async fn new(
        init: Self::Init,
        _input: flume::Sender<Self::Input>,
        _: broadcast::Sender<Self::Output>,
    ) -> Self {
        let connection = init
            .dbus_connection
            .unwrap_or(zbus::Connection::system().await.unwrap());

        let proxy = LoginManagerProxy::new(&connection).await.unwrap();

        Self { proxy }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::PowerOff => drop(self.proxy.power_off(true).await),
            Input::Reboot => drop(self.proxy.reboot(true).await),
            Input::Suspend => drop(self.proxy.suspend(true).await),
            Input::Hibernate => drop(self.proxy.hibernate(true).await),
        }
    }
}
