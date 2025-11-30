use std::collections::HashSet;

use futures_lite::stream::StreamExt;
use tokio::sync::broadcast;
use zbus::fdo::{DBusProxy, NameOwnerChangedStream};
use zbus_names::OwnedBusName;

/// DBus Discovery Service
///
/// Listens for newly created or destroyed DBus objects
#[derive(azalea_derive::StaticHandler)]
pub struct Service {
    stream: NameOwnerChangedStream,
    objects: HashSet<OwnedBusName>,
}

#[derive(Debug)]
pub enum Input {
    QueryObjects(tokio::sync::oneshot::Sender<Vec<OwnedBusName>>),
}

#[derive(Clone, Debug)]
pub enum Output {
    ObjectCreated(OwnedBusName),
    ObjectDeleted(OwnedBusName),
}

impl azalea_service::Service for Service {
    type Init = Option<zbus::Connection>;
    type Input = Input;
    type Event = Output;
    type Output = Output;

    fn handler(init: Self::Init) -> azalea_service::Handler<Self> {
        azalea_service::Handler::new(init, 4, 8)
    }

    async fn new(
        connection: Self::Init,
        _input_sender: flume::Sender<Self::Input>,
        _output_sender: broadcast::Sender<Self::Output>,
    ) -> Self {
        let connection = connection.unwrap_or(zbus::Connection::session().await.unwrap());
        let proxy = DBusProxy::new(&connection).await.unwrap();
        let mut objects: HashSet<OwnedBusName> = Default::default();

        for name in proxy.list_names().await.unwrap_or_default() {
            objects.insert(name);
        }

        let stream = proxy.receive_name_owner_changed().await.unwrap();

        Self { stream, objects }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::QueryObjects(sender) => {
                let names = self.objects.clone().into_iter().collect();
                drop(sender.send(names));
            }
        }
    }

    async fn event_generator(&mut self) -> Self::Event {
        loop {
            if let Some(msg) = self.stream.next().await {
                let Ok(args) = msg.args() else {
                    continue;
                };
                if args.new_owner().is_some() {
                    return Output::ObjectCreated(args.name.into());
                } else {
                    return Output::ObjectDeleted(args.name.into());
                }
            }
        }
    }

    async fn event_handler(
        &mut self,
        event: Self::Event,
        output_sender: &broadcast::Sender<self::Output>,
    ) -> azalea_service::Result<()> {
        output_sender.send(event)?;
        Ok(())
    }
}
