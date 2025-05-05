use relm4::{ComponentSender, Worker};

pub type Output = chrono::DateTime<chrono::Local>;

#[derive(Debug)]
pub struct Model {
    duration: std::time::Duration,
}

impl Worker for Model {
    type Init = std::time::Duration;
    type Input = ();
    type Output = Output;

    fn init(init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self { duration: init }
    }

    fn update(&mut self, _input: (), sender: ComponentSender<Self>) {
        loop {
            let _ = sender.output(chrono::Local::now());
            std::thread::sleep(self.duration);
        }
    }
}
