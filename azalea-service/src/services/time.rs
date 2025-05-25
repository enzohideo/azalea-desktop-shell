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

    fn new(duration: Self::Init) -> Self {
        Self {
            duration,
            ..Default::default()
        }
    }

    fn message(&self, input: Self::Input, _output: &broadcast::Sender<Self::Output>) {
        println!("Received input {input:?}");
    }

    async fn iteration(
        &mut self,
        output: &tokio::sync::broadcast::Sender<Self::Output>,
    ) -> Result<(), error::Error> {
        tokio::time::sleep(self.duration).await;

        let time = chrono::Local::now();

        output.send(Output::Second(time))?;

        {
            let minute = time.minute();

            if self.minute != minute {
                self.minute = minute;
                output.send(Output::Minute(time))?;

                let hour = time.hour();
                if self.hour != hour {
                    self.hour = hour;
                    output.send(Output::Hour(time))?;
                }
            }
        }

        Ok(())
    }
}
