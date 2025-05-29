use azalea_service::{StaticHandler, services};
use gtk::gio::prelude::{ApplicationExt, ApplicationExtManual};

fn main() {
    // TODO: Use struct with named members, then implement Default
    services::time::Service::init(std::time::Duration::from_secs(1));

    let app = relm4::main_application();

    let _keep_service_alive = services::time::Service::listen(|out| {
        println!("hey there {out:?}");
        true
    });
    let _keep_app_alive = app.hold();

    app.connect_activate(|_| {});
    app.run();
}
