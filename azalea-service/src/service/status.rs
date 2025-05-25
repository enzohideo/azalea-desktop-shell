use tokio::sync::mpsc;

use super::Service;

pub enum Status<S>
where
    S: Service,
{
    Started(mpsc::Sender<S::Input>, relm4::JoinHandle<()>),
    Stopped,
}
