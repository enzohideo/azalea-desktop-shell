use azalea_service::{Service, services};
use gtk::gio::prelude::{ApplicationExt, ApplicationExtManual};

fn main() {
    let app = relm4::main_application();
    let _keep_app_alive = app.hold();

    let (tx, rx) = std::sync::mpsc::channel();

    relm4::spawn(async move {
        drop(tx.send(zbus::Connection::session().await.unwrap()));
    });

    let (keep_alive, keep_alive_recv) = std::sync::mpsc::channel();
    relm4::spawn_local(async move {
        let mut handler = services::mpris::Service::handler(Some(rx.recv().unwrap()));

        let _keep_service_alive = handler.listen(|out| {
            println!("hey there {out:?}");
            true
        });

        drop(keep_alive.send(_keep_service_alive));
    });

    app.connect_activate(|_| {});
    app.run();
    drop(keep_alive_recv.recv());
}
