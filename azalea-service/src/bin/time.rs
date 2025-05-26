use azalea_service::{Service, services};
use gtk::gio::prelude::{ApplicationExt, ApplicationExtManual};

fn main() {
    let mut handler = services::time::Service::handler(std::time::Duration::from_secs(1));
    let app = relm4::main_application();

    let _keep_service_alive = handler.listen(|out| {
        println!("hey there {out:?}");
        true
    });
    let _keep_app_alive = app.hold();

    app.connect_activate(|_| {});
    app.run();
}
