use azalea_service::StaticHandler;
use azalea_shell::service;

#[tokio::main]
async fn main() {
    let connection = zbus::Connection::session().await.unwrap();

    service::dbus::mpris::Service::init(service::dbus::mpris::Init {
        dbus_connection: Some(connection),
    });

    service::dbus::mpris::Service::listen(|out| {
        azalea_log::message!("MPRIS output received:\n{out:#?}");
        true
    })
    .join()
    .await;
}
