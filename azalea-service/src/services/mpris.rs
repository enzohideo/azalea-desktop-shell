use std::collections::HashMap;

use futures_lite::stream::StreamExt;
use tokio::sync::broadcast;
use zbus_names::OwnedBusName;

use crate::{
    ListenerHandle, error,
    dbus::media_player2::{Metadata, PlaybackStatus, PlayerProxy},
};

pub struct Service {
    connection: zbus::Connection,
    players: HashMap<OwnedBusName, PlayerProxy<'static>>,
    _listener_handle: Option<ListenerHandle>,
}

#[derive(Clone)]
pub struct Init {
    pub dbus_connection: Option<zbus::Connection>,
    pub dbus_service: Option<crate::Handler<super::dbus::discovery::Service>>,
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

    async fn new(
        init: Self::Init,
        input_sender: broadcast::Sender<Self::Input>,
        _: broadcast::Sender<Self::Output>,
    ) -> Self {
        let connection = init
            .dbus_connection
            .unwrap_or(zbus::Connection::session().await.unwrap());

        let listener_handle = if let Some(mut dbus_service) = init.dbus_service {
            Some(dbus_service.listen(move |output| {
                use super::dbus::discovery::Output;

                match output {
                    Output::ObjectCreated(owned_bus_name) => {
                        if owned_bus_name.contains("org.mpris.MediaPlayer2") {
                            drop(input_sender.send(Input::ObjectCreated(owned_bus_name)));
                        }
                    }
                    Output::ObjectDeleted(owned_bus_name) => {
                        if owned_bus_name.contains("org.mpris.MediaPlayer2") {
                            drop(input_sender.send(Input::ObjectDeleted(owned_bus_name)));
                        }
                    }
                };

                true
            }))
        } else {
            None
        };

        Self {
            _listener_handle: listener_handle,
            connection,
            players: Default::default(),
        }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::ObjectCreated(bus_name) => {
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
