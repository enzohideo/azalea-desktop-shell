use azalea_service::service2::{self, Service};
use chrono::Timelike;
use gtk::{
    gio::prelude::{ApplicationExt, ApplicationExtManual},
    glib::timeout_add_seconds,
};
use tokio::sync::broadcast;

#[derive(Default)]
struct TimeService {
    minute: u32,
    hour: u32,
    duration: std::time::Duration,
}

#[derive(Clone, Debug)]
enum Output {
    Second(chrono::DateTime<chrono::Local>),
    Minute(chrono::DateTime<chrono::Local>),
    Hour(chrono::DateTime<chrono::Local>),
}

impl Service for TimeService {
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

    async fn iteration(&mut self, output: &tokio::sync::broadcast::Sender<Self::Output>) -> Result<(), service2::Error> {
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

fn main() {
    let mut service = TimeService::handler(std::time::Duration::from_millis(500));
    service.start();
    service.listen(|out| {
        println!("hey there {out:?}");
        true
    });
    let app = relm4::main_application();
    let _hold = app.hold();

    app.connect_activate(move |_app| {
        println!("yay");
    });

    timeout_add_seconds(5, move || {
        service.send(format!("fuck you"));
        gtk::glib::ControlFlow::Continue
    });

    app.run();
}
