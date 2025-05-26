use std::collections::HashMap;

use dbus::media_player2::{Metadata, PlayerProxy};
use futures_lite::stream::StreamExt;
use tokio::sync::broadcast;
use zbus::fdo::DBusProxy;
use zbus_names::OwnedBusName;

mod dbus;
use crate::error;

pub struct Service {
    connection: zbus::Connection,
    players: HashMap<OwnedBusName, PlayerProxy<'static>>,
}

#[derive(Clone, Debug)]
pub enum Input {
    NewObject(OwnedBusName),
}

#[derive(Clone, Debug)]
pub enum Event {
    Volume(f64),
    Metadata(Metadata),
}

#[derive(Clone, Debug)]
pub struct Output {
    name: OwnedBusName,
    event: Event,
}

impl crate::Service for Service {
    type Init = Option<zbus::Connection>;
    type Input = Input;
    type Output = Output;

    async fn new(connection: Self::Init, input_sender: broadcast::Sender<Self::Input>) -> Self {
        let connection = connection.unwrap_or(zbus::Connection::session().await.unwrap());
        let proxy = DBusProxy::new(&connection).await.unwrap();

        for name in proxy.list_names().await.unwrap() {
            if name.contains("mpris") {
                println!("{name:?}");
                drop(input_sender.send(Input::NewObject(name)));
            }
        }

        relm4::spawn(async move {
            let mut name_stream = proxy.receive_name_owner_changed().await.unwrap();
            while let Some(msg) = name_stream.next().await {
                if let Ok(args) = msg.args() {
                    if let zbus_names::BusName::WellKnown(name) = &args.name {
                        if name.contains("mpris") {
                            drop(input_sender.send(Input::NewObject(args.name.into())));
                        }
                    }
                }
            }
        });

        Self {
            connection,
            players: Default::default(),
        }
    }

    async fn message(&mut self, input: Self::Input, _output: &broadcast::Sender<Self::Output>) {
        match input {
            Input::NewObject(bus_name) => {
                let proxy =
                    dbus::media_player2::PlayerProxy::new(&self.connection, bus_name.clone())
                        .await
                        .unwrap();
                self.players.insert(bus_name, proxy);
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
                drop(output_sender.send(Output {
                    name: name.clone(),
                    event: Event::Volume(prop.get().await.unwrap()),
                }));
            },
            Some(prop) = metadata.next() => {
                let metadata = prop.get().await.unwrap();
                println!("{:#?}", metadata);
                drop(output_sender.send(Output {
                    name: name.clone(),
                    event: Event::Metadata(metadata),
                }));
            },
            else => continue
        }
    }
}
