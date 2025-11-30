use tokio::sync::broadcast;

#[derive(azalea_derive::StaticHandler)]
pub struct Service {}

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
        if let Some(brightness) = Self::get_brightness() {
            drop(output_sender.send(Output::SystemBrightness(brightness)));
        }
        Self {}
    }

    async fn message(
        &mut self,
        input: Self::Input,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::SystemBrightness(brightness_percent) => {
                blight::set_bl((brightness_percent * 100.) as u32, None).unwrap();
            }
        }
    }
}

impl Service {
    fn get_brightness() -> Option<f64> {
        let dev = blight::Device::new(None).ok()?;
        return Some(dev.current_percent() / 100.);
    }
}
