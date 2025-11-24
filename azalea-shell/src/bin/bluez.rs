use azalea_service::StaticHandler;
use azalea_shell::service;

#[tokio::main]
async fn main() {
    service::dbus::bluez::Service::init(service::dbus::bluez::Init {});

    service::dbus::bluez::Service::listen(|out| {
        azalea_log::message!("BLUEZ output received:\n{out:#?}");
        true
    })
    .join()
    .await;
}
