use std::sync::mpsc;

use clap::Parser;
use gtk::gio::prelude::{ApplicationExt, ApplicationExtManual};

use super::cli::{Arguments, Command, DaemonCommand};

static LOG_NAME: &str = "LilyDesktopShell";
static ID: &str = "usp.ime.LilyDesktopShell";

pub fn run() {
    let args = Arguments::parse();
    let mut gtk_args = vec![std::env::args().next().unwrap()];
    gtk_args.extend(args.gtk_options.clone());

    // TODO: Handle remote commands
    if let Command::Daemon(DaemonCommand::Start) = args.command {
        let app = gtk::Application::builder().application_id(ID).build();
        daemon(&app, &gtk_args);
    } else {
        todo!();
    }
}

fn daemon(app: &gtk::Application, gtk_args: &Vec<String>) {
    let (ping_tx, ping_rx) = mpsc::channel();
    let (pong_tx, pong_rx) = mpsc::channel();

    ping_tx.send(app.hold()).expect("Daemon could not ping!");

    app.connect_activate(move |_app| {
        if let Ok(app_guard) = ping_rx.try_recv() {
            gtk::glib::g_message!(LOG_NAME, "Daemon has started");

            pong_tx.send(app_guard).expect("Daemon could not pong!");
        }
    });

    app.run_with_args(gtk_args);

    drop(pong_rx.try_recv());
}
