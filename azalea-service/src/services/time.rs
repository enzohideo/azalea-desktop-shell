use relm4::ComponentSender;

use super::WorkerExt;

pub type Output = chrono::DateTime<chrono::Local>;

// TODO: macro to create this and the type aliases
pub struct State {
    duration: std::time::Duration,
}
pub type Model = super::Model<State>;
pub type Service = crate::Service<Model>;

// TODO: Maybe apply this trait to Model directly
impl WorkerExt<Model> for State {
    type Init = std::time::Duration;
    type Output = Output;

    fn new(init: Self::Init) -> Self {
        Self { duration: init }
    }

    async fn update(&mut self, sender: &ComponentSender<Model>) {
        sender.output(chrono::Local::now()).unwrap_or(());
        tokio::time::sleep(self.duration).await;
    }
}
