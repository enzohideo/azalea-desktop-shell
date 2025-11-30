use tokio::sync::broadcast;

#[derive(azalea_derive::StaticHandler)]
pub struct Service {
    client: open_meteo_rs::Client,
}

#[derive(Clone, Default)]
pub struct Init {}

#[derive(Clone, Debug)]
pub enum Input {
    Update,
}

#[derive(Clone, Debug)]
pub enum Output {
    Temperature((f64, String)),
}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = Input;
    type Event = ();
    type Output = Output;

    const DISABLE_EVENTS: bool = true;

    async fn new(
        _init: Self::Init,
        sender: flume::Sender<Self::Input>,
        _output_sender: broadcast::Sender<Self::Output>,
    ) -> Self {
        let client = open_meteo_rs::Client::new();

        drop(sender.send(Input::Update));

        Self { client }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::Update => {
                if let Some(temperature) = self.get_temperature().await {
                    drop(output_sender.send(Output::Temperature(temperature)));
                }
            }
        }
    }
}

impl Service {
    async fn get_temperature(&self) -> Option<(f64, String)> {
        let mut opts = open_meteo_rs::forecast::Options::default();

        opts.location = open_meteo_rs::Location {
            lat: -23.5558,
            lng: -46.6396,
        };

        opts.temperature_unit = Some(open_meteo_rs::forecast::TemperatureUnit::Celsius); // or
        opts.wind_speed_unit = Some(open_meteo_rs::forecast::WindSpeedUnit::Kmh); // or
        opts.precipitation_unit = Some(open_meteo_rs::forecast::PrecipitationUnit::Millimeters); // or
        opts.current.push("temperature_2m".into());

        let res = self.client.forecast(opts).await.unwrap();
        res.current.and_then(|res| {
            if let Some(temp) = res.values.get("temperature_2m") {
                Some((
                    match temp.value.clone() {
                        serde_json::Value::Number(number) => number.as_f64().unwrap_or(0.),
                        _ => 0.,
                    },
                    temp.unit.clone().unwrap_or(format!("C°")),
                ))
            } else {
                None
            }
        })
    }
}
