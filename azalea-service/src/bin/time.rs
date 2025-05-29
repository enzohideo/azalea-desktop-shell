use azalea_service::{StaticHandler, services};
use gtk::gio::prelude::{ApplicationExt, ApplicationExtManual};

fn main() {
    let app = relm4::main_application();

    let _keep_service_alive = services::time::Service::listen(|out| {
        println!("hey there {out:?}");
        true
    });
    let _keep_app_alive = app.hold();

    app.connect_activate(|_| {});
    app.run();
}
