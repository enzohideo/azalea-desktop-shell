use std::collections::HashMap;

use futures_lite::stream::StreamExt;
use tokio::sync::{broadcast, oneshot};
use zbus_names::OwnedBusName;

use crate::{
    ListenerHandle, StaticHandler,
    dbus::mpris::media_player2::{Metadata, PlaybackStatus, PlayerProxy},
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
pub enum Input {
    ObjectCreated(OwnedBusName),
    ObjectDeleted(OwnedBusName),
}

#[derive(Clone, Debug)]
pub enum Event {
    Volume(f64),
    Metadata(Metadata),
    Position(i64),
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
        _output_sender: &broadcast::Sender<Self::Output>,
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

    loop {
        tokio::select! {
            Some(prop) = volume.next() => {
                let volume = prop.get().await.unwrap();
                azalea_log::debug!("[MPRIS] Volume changed for object {}: {}", name, volume);
                drop(output_sender.send(Output {
                    name: name.clone(),
                    event: Event::Volume(volume),
                }));
            },
            Some(prop) = metadata.next() => {
                let metadata = prop.get().await.unwrap();
                azalea_log::debug!("[MPRIS] Metadata changed for object {}: {:#?}", name, metadata);
                drop(output_sender.send(Output {
                    name: name.clone(),
                    event: Event::Metadata(metadata),
                }));
            },
            () = async {
                // TODO: Check if it's better/easier to use Rate on widget client or just poll here
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                while let Ok(playback_status) = player.playback_status().await {
                    match playback_status {
                        PlaybackStatus::Playing => {
                            azalea_log::debug!("Playback status {:?}", playback_status);
                            drop(output_sender.send(Output {
                                name: name.clone(),
                                event: Event::Position(player.position().await.unwrap()),
                            }));
                        },
                        PlaybackStatus::Paused | PlaybackStatus::Stopped => {
                            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                        },
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            } => (),
            else => continue
        }
    }
}

crate::impl_static_handler!(Service);
