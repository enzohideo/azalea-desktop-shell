use chrono::Timelike;
use tokio::sync::broadcast;

use crate::error;

#[derive(Default)]
pub struct Service {
    minute: u32,
    hour: u32,
    duration: std::time::Duration,
}

#[derive(Clone, Debug)]
pub enum Output {
    Second(chrono::DateTime<chrono::Local>),
    Minute(chrono::DateTime<chrono::Local>),
    Hour(chrono::DateTime<chrono::Local>),
}

impl crate::Service for Service {
    type Init = std::time::Duration;
    type Input = String;
    type Output = Output;

    async fn new(
        duration: Self::Init,
        _: broadcast::Sender<Self::Input>,
        _: broadcast::Sender<Self::Output>,
    ) -> Self {
        Self {
            duration,
            ..Default::default()
        }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) {
        println!("Received input {input:?}");
    }

    async fn iteration(
        &mut self,
        output_sender: &tokio::sync::broadcast::Sender<Self::Output>,
    ) -> Result<(), error::Error> {
        tokio::time::sleep(self.duration).await;

        let time = chrono::Local::now();

        output_sender.send(Output::Second(time))?;

        {
            let minute = time.minute();

            if self.minute != minute {
                self.minute = minute;
                output_sender.send(Output::Minute(time))?;

                let hour = time.hour();
                if self.hour != hour {
                    self.hour = hour;
                    output_sender.send(Output::Hour(time))?;
                }
            }
        }

        Ok(())
    }
}

crate::impl_static_handler!(Service);
