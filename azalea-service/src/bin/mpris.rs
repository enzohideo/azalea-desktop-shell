use azalea_service::{Service, services};
use gtk::gio::prelude::{ApplicationExt, ApplicationExtManual};

fn main() {
    let app = relm4::main_application();
    let _keep_app_alive = app.hold();

    let (tx, rx) = std::sync::mpsc::channel();

    relm4::spawn(async move {
        drop(tx.send(zbus::Connection::session().await.unwrap()));
    });

    let (keep_alive_listener, keep_alive_listener_recv) = std::sync::mpsc::channel();
    relm4::spawn_local(async move {
        let connection = rx.recv().unwrap();
        let dbus_handler = services::dbus::discovery::Service::handler(Some(connection.clone()));

        let mut mpris_handler = services::mpris::Service::handler(services::mpris::Init {
            dbus_service: Some(dbus_handler),
            dbus_connection: Some(connection),
        });

        let _keep_service_alive = mpris_handler.listen(|out| {
            azalea_log::message!("mpris service output: {out:#?}");
            true
        });

        drop(keep_alive_listener.send(_keep_service_alive));
    });

    app.connect_activate(|_| {});
    app.run();
    drop(keep_alive_listener_recv.recv());
}
