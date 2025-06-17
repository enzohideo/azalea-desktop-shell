use std::collections::HashMap;

use futures_lite::StreamExt;
use tokio::sync::broadcast;

#[derive(azalea_derive::StaticHandler)]
pub struct Service {
    session: bluer::Session,
}

#[derive(Default, Debug)]
pub struct Device {
    name: Option<String>,
}

#[derive(Default, Clone)]
pub struct Init {
    pub dbus_connection: Option<zbus::Connection>,
}

pub type AdapterName = String;

#[derive(Clone, Debug)]
pub enum Input {
    Adapters(flume::Sender<Vec<AdapterName>>),
    Devices(flume::Sender<HashMap<String, HashMap<String, Device>>>),
}

pub enum Event {}

#[derive(Clone, Debug)]
pub enum Output {}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = Input;
    type Event = ();
    type Output = Output;
    const DISABLE_EVENTS: bool = true;

    async fn new(
        _init: Self::Init,
        _input: flume::Sender<Self::Input>,
        _: broadcast::Sender<Self::Output>,
    ) -> Self {
        let session = bluer::Session::new().await.unwrap();

        Self { session }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::Adapters(sender) => {
                let names = self.session.adapter_names().await.unwrap_or_default();
                drop(sender.send(names));
            }
            Input::Devices(sender) => {
                let devices = futures_lite::stream::iter(
                    self.session.adapter_names().await.unwrap_or_default(),
                )
                .then(|name| {
                    let name = name.clone();
                    let session = self.session.clone();
                    async move {
                        (
                            name.to_owned(),
                            match session.adapter(&name) {
                                Ok(adapter) => {
                                    futures_lite::stream::iter(
                                        adapter.device_addresses().await.unwrap_or_default(),
                                    )
                                    .then(|addr| {
                                        let adapter = adapter.clone();
                                        async move {
                                            (
                                                addr.to_string(),
                                                match adapter.device(addr) {
                                                    Ok(device) => Device {
                                                        name: device.name().await.unwrap_or(None),
                                                    },
                                                    Err(e) => {
                                                        azalea_log::warning::<Self>(
                                                            &format!("Failed to get device from adapter {addr}: {e:?}"),
                                                        );
                                                        Default::default()
                                                    },
                                                },
                                            )
                                        }
                                    })
                                    .collect::<HashMap<String, Device>>()
                                    .await
                                }
                                Err(e) => {
                                    azalea_log::warning::<Self>(
                                        &format!("Failed to get adapter with name {name}: {e:?}"),
                                    );
                                    Default::default()
                                }
                            },
                        )
                    }
                })
                .collect()
                .await;
                drop(sender.send(devices));
            }
        }
    }

    async fn event_generator(&mut self) -> Self::Event {}

    async fn event_handler(
        &mut self,
        _event: Self::Event,
        _output_sender: &tokio::sync::broadcast::Sender<Self::Output>,
    ) -> azalea_service::Result<()> {
        Ok(())
    }
}
