use std::sync::mpsc;

use gtk::gio::prelude::{ApplicationExt, ApplicationExtManual};

static LOG_NAME: &str = "LilyDesktopShell";
static ID: &str = "usp.ime.LilyDesktopShell";

pub fn run() {
    let app = gtk::Application::builder().application_id(ID).build();

    // TODO: Handle remote commands
    daemon(&app);
}

fn daemon(app: &gtk::Application) {
    let (ping_tx, ping_rx) = mpsc::channel();
    let (pong_tx, pong_rx) = mpsc::channel();

    ping_tx.send(app.hold()).expect("Daemon could not ping!");

    app.connect_activate(move |_app| {
        if let Ok(app_guard) = ping_rx.try_recv() {
            gtk::glib::g_message!(LOG_NAME, "Daemon has started");

            pong_tx.send(app_guard).expect("Daemon could not pong!");
        }
    });

    app.run();

    drop(pong_rx.try_recv());
}
