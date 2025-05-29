use azalea_service::{StaticHandler, services};

#[tokio::main]
async fn main() {
    let connection = zbus::Connection::session().await.unwrap();

    services::dbus::mpris::Service::init(services::dbus::mpris::Init {
        dbus_connection: Some(connection),
    });

    services::dbus::mpris::Service::listen(|out| {
        azalea_log::message!("MPRIS output received:\n{out:#?}");
        true
    })
    .join()
    .await;
}
