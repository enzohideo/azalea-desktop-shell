use std::collections::HashSet;

use futures_lite::stream::StreamExt;
use tokio::sync::broadcast;
use zbus::fdo::DBusProxy;
use zbus_names::OwnedBusName;

use crate::error;

/// DBus Discovery Service
///
/// Listens for newly created or destroyed DBus objects
#[derive(Clone)]
pub struct Service {
    proxy: DBusProxy<'static>,
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

impl crate::Service for Service {
    type Init = Option<zbus::Connection>;
    type Input = Input;
    type Output = Output;

    fn handler(init: Self::Init) -> crate::Handler<Self> {
        crate::Handler::new(init, 1, 8)
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

        Self { proxy, objects }
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

    async fn iteration(
        &mut self,
        output_sender: &broadcast::Sender<self::Output>,
    ) -> Result<(), error::Error> {
        let mut name_stream = self.proxy.receive_name_owner_changed().await.unwrap();

        while let Some(msg) = name_stream.next().await {
            let Ok(args) = msg.args() else {
                continue;
            };
            if args.new_owner().is_some() {
                output_sender.send(Output::ObjectCreated(args.name.into()))?;
            } else {
                output_sender.send(Output::ObjectDeleted(args.name.into()))?;
            }
        }

        Ok(())
    }
}

crate::impl_static_handler!(Service);
