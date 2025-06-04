use azalea_service::StaticHandler;
use azalea_shell::service;

#[tokio::main]
async fn main() {
    let connection = zbus::Connection::system().await.unwrap();

    service::dbus::network_manager::Service::init(service::dbus::network_manager::Init {
        dbus_connection: Some(connection),
    });

    service::dbus::network_manager::Service::listen(|out| {
        azalea_log::message!("NETWORK output received:\n{out:#?}");
        true
    })
    .join()
    .await;
}
