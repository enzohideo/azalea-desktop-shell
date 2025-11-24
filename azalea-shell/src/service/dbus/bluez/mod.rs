use std::collections::HashMap;

use bluer::DeviceEvent;
use futures_lite::StreamExt;
use tokio::sync::broadcast;

#[derive(azalea_derive::StaticHandler)]
pub struct Service {
    session: bluer::Session,
    adapter: bluer::Adapter,
    devices: HashMap<String, bluer::Device>,
}

#[derive(Default, Clone, Debug)]
pub struct Device {
    pub address: String,
    pub name: Option<String>,
    pub is_connected: bool,
}

#[derive(Default, Clone)]
pub struct Init {}

pub type AdapterName = String;
pub type DeviceAdress = String;

#[derive(Clone, Debug)]
pub enum Input {
    Adapters(flume::Sender<Vec<AdapterName>>),
    Devices(flume::Sender<HashMap<String, Device>>),
    Connect(DeviceAdress, bool),
}

pub enum Event {}

#[derive(Clone, Debug)]
pub enum Output {
    Connected(DeviceAdress, bool),
}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = Input;
    type Event = ();
    type Output = Output;
    const DISABLE_EVENTS: bool = true;

    async fn new(
        _init: Self::Init,
        _input: flume::Sender<Self::Input>,
        output_sender: broadcast::Sender<Self::Output>,
    ) -> Self {
        let session = bluer::Session::new().await.unwrap();
        let adapter = session.default_adapter().await.unwrap();
        let devices =
            futures_lite::stream::iter(adapter.device_addresses().await.unwrap_or_default())
                .then(|addr| {
                    let adapter = adapter.clone();
                    let device = adapter.device(addr).unwrap();
                    let device_clone = device.clone();
                    let sender = output_sender.clone();
                    relm4::spawn(async move {
                        listen_to_player(device_clone, sender).await;
                    });
                    async move { (addr.to_string(), device) }
                })
                .collect::<HashMap<String, bluer::Device>>()
                .await;

        Self {
            session,
            adapter,
            devices,
        }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::Adapters(sender) => {
                let names = self.session.adapter_names().await.unwrap_or_default();
                drop(sender.send(names));
            }
            Input::Devices(sender) => drop(
                sender.send(
                    futures_lite::stream::iter(self.devices.iter())
                        .then(|(address, device)| async move {
                            let address = address.to_string();
                            (
                                address.clone(),
                                Device {
                                    address,
                                    name: device.name().await.unwrap_or(None),
                                    is_connected: device.is_connected().await.unwrap_or(false),
                                },
                            )
                        })
                        .collect::<HashMap<String, Device>>()
                        .await,
                ),
            ),
            Input::Connect(device_address, connect) => match self.devices.get(&device_address) {
                Some(device) => {
                    if connect {
                        if let Ok(_) = device.connect().await {
                            drop(output_sender.send(Output::Connected(device_address, connect)));
                        }
                    } else {
                        if let Ok(_) = device.disconnect().await {
                            drop(output_sender.send(Output::Connected(device_address, connect)));
                        }
                    }
                }
                None => todo!(),
            },
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

async fn listen_to_player(device: bluer::Device, output_sender: broadcast::Sender<Output>) {
    let address = device.address().to_string();
    let mut event_stream = device.events().await.unwrap();
    while let Some(event) = event_stream.next().await {
        match event {
            DeviceEvent::PropertyChanged(device_property) => match device_property {
                bluer::DeviceProperty::Name(_) => {}
                bluer::DeviceProperty::RemoteAddress(_address) => {}
                bluer::DeviceProperty::AddressType(_address_type) => {}
                bluer::DeviceProperty::Icon(_) => {}
                bluer::DeviceProperty::Class(_) => {}
                bluer::DeviceProperty::Appearance(_) => {}
                bluer::DeviceProperty::Uuids(_hash_set) => {}
                bluer::DeviceProperty::Paired(_) => {}
                bluer::DeviceProperty::Connected(connected) => {
                    drop(output_sender.send(Output::Connected(address.clone(), connected)));
                }
                bluer::DeviceProperty::Trusted(_) => {}
                bluer::DeviceProperty::Blocked(_) => {}
                bluer::DeviceProperty::WakeAllowed(_) => {}
                bluer::DeviceProperty::Alias(_) => {}
                bluer::DeviceProperty::LegacyPairing(_) => {}
                bluer::DeviceProperty::Modalias(_modalias) => {}
                bluer::DeviceProperty::Rssi(_) => {}
                bluer::DeviceProperty::TxPower(_) => {}
                bluer::DeviceProperty::ManufacturerData(_hash_map) => {}
                bluer::DeviceProperty::ServiceData(_hash_map) => {}
                bluer::DeviceProperty::ServicesResolved(_) => {}
                bluer::DeviceProperty::AdvertisingFlags(_items) => {}
                bluer::DeviceProperty::AdvertisingData(_hash_map) => {}
                bluer::DeviceProperty::BatteryPercentage(_) => {}
                _ => {}
            },
        }
    }
}
