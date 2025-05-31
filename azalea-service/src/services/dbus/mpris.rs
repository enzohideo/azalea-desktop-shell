use std::collections::HashMap;

use futures_lite::stream::StreamExt;
use tokio::sync::{broadcast, oneshot};
pub use zbus_names::OwnedBusName;

use crate::{
    ListenerHandle, StaticHandler,
    dbus::mpris::media_player2::{Metadata, PlaybackRate, PlaybackStatus, PlayerProxy},
    error,
};

pub struct Service {
    connection: zbus::Connection,
    players: HashMap<OwnedBusName, PlayerProxy<'static>>,
    _listener_handle: ListenerHandle,
}

#[derive(Default, Clone)]
pub struct Init {
    pub dbus_connection: Option<zbus::Connection>,
}

#[derive(Clone, Debug)]
pub enum Action {
    PlayPause(OwnedBusName),
    Previous(OwnedBusName),
    Next(OwnedBusName),
}

#[derive(Clone, Debug)]
pub enum Input {
    ObjectCreated(OwnedBusName),
    ObjectDeleted(OwnedBusName),
    QueryPosition(OwnedBusName),
    Action(Action),
}

#[derive(Clone, Debug)]
pub enum Event {
    Volume(f64),
    Position(i64),
    Metadata(Metadata),
    PlaybackStatus(PlaybackStatus),
    PlaybackRate(PlaybackRate),
}

#[derive(Clone, Debug)]
pub struct Output {
    pub name: OwnedBusName,
    pub event: Event,
}

impl crate::Service for Service {
    type Init = Init;
    type Input = Input;
    type Output = Output;

    fn handler(init: Self::Init) -> crate::Handler<Self> {
        crate::Handler::new(init, 1, 8)
    }

    async fn new(
        init: Self::Init,
        input_sender: flume::Sender<Self::Input>,
        output_sender: broadcast::Sender<Self::Output>,
    ) -> Self {
        let connection = init
            .dbus_connection
            .unwrap_or(zbus::Connection::session().await.unwrap());

        let listener_handle =
            super::discovery::Service::filtered_forward(input_sender.into(), |output| {
                use super::discovery::Output;

                match output {
                    Output::ObjectCreated(owned_bus_name) => {
                        if owned_bus_name.contains("org.mpris.MediaPlayer2") {
                            return Some(Input::ObjectCreated(owned_bus_name));
                        }
                    }
                    Output::ObjectDeleted(owned_bus_name) => {
                        if owned_bus_name.contains("org.mpris.MediaPlayer2") {
                            return Some(Input::ObjectDeleted(owned_bus_name));
                        }
                    }
                };

                None
            });

        let mut service = Self {
            _listener_handle: listener_handle,
            connection,
            players: Default::default(),
        };

        let (tx, rx) = oneshot::channel();
        super::discovery::Service::send(super::discovery::Input::QueryObjects(tx));
        match rx.await {
            Ok(names) => {
                for name in names {
                    if !name.contains("org.mpris.MediaPlayer2") {
                        continue;
                    }
                    service
                        .message(Input::ObjectCreated(name), &output_sender)
                        .await;
                }
            }
            Err(e) => azalea_log::debug!("Failed to send: {e}"),
        }

        service
    }

    async fn message(
        &mut self,
        input: Self::Input,
        output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::ObjectCreated(bus_name) => {
                if self.players.contains_key(&bus_name) {
                    return;
                }
                azalea_log::debug!("[MPRIS] Object created: {}", bus_name);
                let proxy = PlayerProxy::new(&self.connection, bus_name.clone())
                    .await
                    .unwrap();
                self.players.insert(bus_name, proxy);
            }
            Input::ObjectDeleted(bus_name) => {
                azalea_log::debug!("[MPRIS] Object deleted: {}", bus_name);
                self.players.remove(&bus_name);
            }
            Input::QueryPosition(bus_name) => {
                let Some(player) = self.players.get(&bus_name) else {
                    return;
                };
                let position = player.position().await.unwrap_or(0);
                drop(output_sender.send(Output {
                    name: bus_name,
                    event: Event::Position(position),
                }));
            }
            Input::Action(action) => {
                azalea_log::debug!("[MPRIS] Triggered action: {:?}", action);
                // TODO: return anyhow error
                match action {
                    Action::PlayPause(bus_name) => {
                        drop(
                            self.players
                                .get(&bus_name)
                                .map(|p| p.play_pause())
                                .unwrap()
                                .await,
                        );
                    }
                    Action::Previous(bus_name) => {
                        drop(
                            self.players
                                .get(&bus_name)
                                .map(|p| p.previous())
                                .unwrap()
                                .await,
                        );
                    }
                    Action::Next(bus_name) => {
                        drop(self.players.get(&bus_name).map(|p| p.next()).unwrap().await);
                    }
                }
            }
        }
    }

    async fn iteration(
        &mut self,
        output_sender: &broadcast::Sender<self::Output>,
    ) -> Result<(), error::Error> {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let mut players = tokio::task::JoinSet::new();

        for (name, player) in self.players.clone() {
            players.spawn(listen_to_player(name, player, output_sender.clone()));
        }

        while let Some(_) = players.join_next().await {}

        Ok(())
    }
}

async fn listen_to_player<'a>(
    name: OwnedBusName,
    player: PlayerProxy<'a>,
    output_sender: broadcast::Sender<Output>,
) {
    let mut volume = player.receive_volume_changed().await;
    let mut metadata = player.receive_metadata_changed().await;
    let mut playback_status = player.receive_playback_status_changed().await;
    let mut playback_rate = player.receive_rate_changed().await;

    loop {
        tokio::select! {
            Some(prop) = volume.next() => {
                let Ok(value) = prop.get().await else { continue; };
                azalea_log::debug!("[MPRIS] Volume changed for object {}: {}", name, value);
                drop(output_sender.send(Output {
                    name: name.clone(),
                    event: Event::Volume(value),
                }));
            },
            Some(prop) = metadata.next() => {
                let Ok(value) = prop.get().await else { continue; };
                azalea_log::debug!("[MPRIS] Metadata changed for object {}: {:#?}", name, value);
                drop(output_sender.send(Output {
                    name: name.clone(),
                    event: Event::Metadata(value),
                }));
            },
            Some(prop) = playback_status.next() => {
                let Ok(value) = prop.get().await else { continue; };
                azalea_log::debug!("[MPRIS] PlaybackStatus changed for object {}: {:#?}", name, value);
                drop(output_sender.send(Output {
                    name: name.clone(),
                    event: Event::PlaybackStatus(value),
                }));
                let Ok(position) = player.position().await else { continue; };
                drop(output_sender.send(Output {
                    name: name.clone(),
                    event: Event::Position(position),
                }));
            },
            Some(prop) = playback_rate.next() => {
                let Ok(value) = prop.get().await else { continue; };
                azalea_log::debug!("[MPRIS] PlaybackRate changed for object {}: {:#?}", name, value);
                drop(output_sender.send(Output {
                    name: name.clone(),
                    event: Event::PlaybackRate(value),
                }));
            },
            else => continue
        }
    }
}

crate::impl_static_handler!(Service);
