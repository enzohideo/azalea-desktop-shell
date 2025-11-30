use brightness::{Brightness, BrightnessDevice};
use futures_lite::StreamExt;
use std::env;

#[tokio::main]
async fn main() {
    let percentage = env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .expect("Desired brightness percentage must be given as parameter");
    run(percentage).await;
}

async fn run(percentage: u32) {
    let devices: Vec<BrightnessDevice> = brightness::brightness_devices()
        .filter_map(|dev| dev.ok())
        .collect()
        .await;

    for mut dev in devices {
        drop(show_brightness(&dev).await);
        drop(dev.set(percentage).await);
        drop(show_brightness(&dev).await);
    }
}

async fn show_brightness(dev: &BrightnessDevice) -> Result<(), brightness::Error> {
    println!(
        "Brightness of device {} is {}%",
        dev.friendly_device_name().await?,
        dev.get().await?
    );
    Ok(())
}
