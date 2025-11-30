pub mod service;

use std::collections::HashMap;

use azalea_service::StaticHandler;
use tokio::sync::{broadcast, oneshot};

#[derive(azalea_derive::StaticHandler)]
pub struct Service {
    notifications: HashMap<u32, service::Notification>,
    conn: Option<zbus::Connection>,
    rx: flume::Receiver<service::Event>,
}

pub struct Streams {}

#[derive(Default, Clone)]
pub struct Init {}

#[derive(Clone, Debug)]
pub enum Input {
    Close,
}

pub enum Event {
    Notifications(service::Event),
}

#[derive(Clone, Debug)]
pub enum Output {
    Notification(service::Notification),
}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = Input;
    type Event = Event;
    type Output = Output;

    fn handler(init: Self::Init) -> azalea_service::Handler<Self> {
        azalea_service::Handler::new(init, 16, 16)
    }

    async fn new(
        _init: Self::Init,
        _input: flume::Sender<Self::Input>,
        _output_sender: broadcast::Sender<Self::Output>,
    ) -> Self {
        let (tx, rx) = flume::unbounded();
        let (discovery_tx, discovery_rx) = oneshot::channel();
        super::discovery::Service::send(super::discovery::Input::QueryObjects(discovery_tx));
        let conn = match discovery_rx.await {
            Ok(names) => {
                let mut notification_server_found = false;

                for name in names {
                    if name.contains("org.freedesktop.Notifications") {
                        notification_server_found = true;
                        break;
                    }
                }

                if notification_server_found {
                    None
                } else {
                    let notifications = service::Notifications::new(tx);

                    if let Ok(conn) = zbus::conn::Builder::session()
                        .and_then(|conn| conn.name("org.freedesktop.Notifications"))
                        .and_then(|conn| {
                            conn.serve_at("/org/freedesktop/Notifications", notifications)
                        })
                        .map(|conn| conn.build())
                    {
                        conn.await.ok()
                    } else {
                        azalea_log::warning!("There's already a notification server running!");
                        None
                    }
                }
            }
            Err(e) => {
                azalea_log::warning!("Failed to query for other notification servers: {e}");
                None
            }
        };

        Self {
            notifications: Default::default(),
            conn,
            rx,
        }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::Close => todo!(),
        }
    }

    async fn event_generator(&mut self) -> Self::Event {
        loop {
            if self.conn.is_none() {
                // TODO: Try to recreate?
                tokio::time::sleep(std::time::Duration::from_secs(100)).await;
            } else {
                if let Ok(event) = self.rx.recv_async().await {
                    return Event::Notifications(event);
                }
            }
        }
    }

    async fn event_handler(
        &mut self,
        event: Self::Event,
        output_sender: &tokio::sync::broadcast::Sender<Self::Output>,
    ) -> azalea_service::Result<()> {
        match event {
            Event::Notifications(event) => match event {
                service::Event::Notify(notification) => {
                    self.notifications
                        .insert(notification.id, notification.clone());
                    drop(output_sender.send(Output::Notification(notification)))
                }
            },
        }
        Ok(())
    }
}
