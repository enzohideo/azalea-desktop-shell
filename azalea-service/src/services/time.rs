use relm4::{ComponentSender, Worker};

pub type Output = chrono::DateTime<chrono::Local>;

#[derive(Debug)]
pub struct Model {
    duration: std::time::Duration,
}

impl Worker for Model {
    type Init = std::time::Duration;
    type Input = crate::Input;
    type Output = Output;

    fn init(init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self { duration: init }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>) {
        // TODO: Use a Command instead of blocking here
        match input {
            crate::Input::Start => loop {
                let _ = sender.output(chrono::Local::now());
                std::thread::sleep(self.duration);
            },
            crate::Input::Stop => todo!(),
        }
    }
}
