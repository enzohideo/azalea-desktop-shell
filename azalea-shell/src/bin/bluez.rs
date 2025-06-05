use azalea_service::StaticHandler;
use azalea_shell::service;

#[tokio::main]
async fn main() {
    let connection = zbus::Connection::system().await.unwrap();

    service::dbus::bluez::Service::init(service::dbus::bluez::Init {
        dbus_connection: Some(connection),
    });

    service::dbus::bluez::Service::listen(|out| {
        azalea_log::message!("BLUEZ output received:\n{out:#?}");
        true
    })
    .join()
    .await;
}
