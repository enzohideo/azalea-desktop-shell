use chrono::Timelike;
use tokio::sync::broadcast;

use azalea_service::error;

#[derive(Default)]
pub struct Service {
    minute: u32,
    hour: u32,
    interval_duration: std::time::Duration,
}

#[derive(Clone)]
pub struct Init {
    interval_duration: std::time::Duration,
}

impl Default for Init {
    fn default() -> Self {
        Self {
            interval_duration: std::time::Duration::from_secs(1),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Output {
    Second(chrono::DateTime<chrono::Local>),
    Minute(chrono::DateTime<chrono::Local>),
    Hour(chrono::DateTime<chrono::Local>),
}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = String;
    type Event = ();
    type Output = Output;

    async fn new(
        init: Self::Init,
        _: flume::Sender<Self::Input>,
        _: broadcast::Sender<Self::Output>,
    ) -> Self {
        Self {
            interval_duration: init.interval_duration,
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

    async fn event_generator(&mut self) -> Self::Event {
        tokio::time::sleep(self.interval_duration).await;
    }

    async fn event_handler(
        &mut self,
        _event: Self::Event,
        output_sender: &tokio::sync::broadcast::Sender<Self::Output>,
    ) -> Result<(), error::Error> {
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

azalea_service::impl_static_handler!(Service);
