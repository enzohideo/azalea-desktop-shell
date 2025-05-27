use futures_lite::stream::StreamExt;
use tokio::sync::broadcast;
use zbus::fdo::DBusProxy;
use zbus_names::OwnedBusName;

use crate::error;

#[derive(Clone)]
pub struct Service {
    proxy: DBusProxy<'static>,
}

#[derive(Clone, Debug)]
pub enum Input {
    QueryObjects, // TODO: Add channel to receive response
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

    async fn new(
        connection: Self::Init,
        _input_sender: broadcast::Sender<Self::Input>,
        output_sender: broadcast::Sender<Self::Output>,
    ) -> Self {
        let connection = connection.unwrap_or(zbus::Connection::session().await.unwrap());
        let proxy = DBusProxy::new(&connection).await.unwrap();

        for name in proxy.list_names().await.unwrap() {
            drop(output_sender.send(Output::ObjectCreated(name)));
        }

        Self { proxy }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::QueryObjects => todo!(),
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
