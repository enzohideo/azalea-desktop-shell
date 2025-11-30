use brightness::{Brightness, BrightnessDevice};
use futures_lite::StreamExt;
use tokio::sync::broadcast;

#[derive(azalea_derive::StaticHandler)]
pub struct Service {
    tx: flume::Sender<f64>,
}

#[derive(Clone, Default)]
pub struct Init {}

#[derive(Clone, Debug)]
pub enum Input {
    /// Default sink brightness
    SystemBrightness(f64),
}

#[derive(Clone, Debug)]
pub enum Output {
    /// Default sink brightness
    SystemBrightness(f64),
}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = Input;
    type Event = ();
    type Output = Output;

    const DISABLE_EVENTS: bool = true;

    async fn new(
        _init: Self::Init,
        _: flume::Sender<Self::Input>,
        output_sender: broadcast::Sender<Self::Output>,
    ) -> Self {
        let mut devices: Vec<BrightnessDevice> = brightness::brightness_devices()
            .filter_map(|dev| dev.ok())
            .collect()
            .await;

        if let Some(dev) = devices.first() {
            let brightness = dev.get().await.unwrap_or(0);
            drop(output_sender.send(Output::SystemBrightness(brightness as f64 / 100.)));
        }

        let (tx, rx) = flume::unbounded();

        relm4::spawn(async move {
            while let Ok(brightness_percent) = rx.recv_async().await {
                for dev in &mut devices {
                    drop(dev.set((brightness_percent * 100.) as u32).await);
                }
            }
        });

        Self { tx }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::SystemBrightness(brightness_percent) => {
                drop(output_sender.send(Output::SystemBrightness(brightness_percent)));
                // TODO: Debounce
                let _ = self.tx.send_async(brightness_percent).await;
            }
        }
    }
}
