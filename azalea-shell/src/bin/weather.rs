#[tokio::main]
async fn main() {
    let client = open_meteo_rs::Client::new();
    let mut opts = open_meteo_rs::forecast::Options::default();

    opts.location = open_meteo_rs::Location {
        lat: -23.5558,
        lng: -46.6396,
    };

    opts.temperature_unit = Some(open_meteo_rs::forecast::TemperatureUnit::Celsius); // or
    opts.wind_speed_unit = Some(open_meteo_rs::forecast::WindSpeedUnit::Kmh); // or
    opts.precipitation_unit = Some(open_meteo_rs::forecast::PrecipitationUnit::Millimeters); // or
    opts.current.push("temperature_2m".into());

    let res = client.forecast(opts).await.unwrap();
    let temperature = res.current.and_then(|res| {
        if let Some(temp) = res.values.get("temperature_2m") {
            Some((
                temp.unit.clone().unwrap_or(format!("C°")),
                temp.value.clone(),
            ))
        } else {
            None
        }
    });

    println!("{:#?}", temperature);
}
