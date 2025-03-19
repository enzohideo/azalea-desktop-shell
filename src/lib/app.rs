use std::{
    os::unix::net::{UnixListener, UnixStream},
    sync::mpsc,
};

use clap::Parser;
use gtk::gio::{
    self,
    prelude::{ApplicationExt, ApplicationExtManual},
};

use crate::{cli::RemoteCommand, socket::UnixStreamWrapper};

use super::cli::{Arguments, Command};

static LOG_NAME: &str = "LilyDesktopShell";
static ID: &str = "usp.ime.LilyDesktopShell";

pub fn run() {
    let args = Arguments::parse();
    let mut gtk_args = vec![std::env::args().next().unwrap()];
    gtk_args.extend(args.gtk_options.clone());

    // TODO: Handle remote commands
    // TODO: Check if it's remote through dbus
    match args.command {
        Command::Daemon => daemon(&gtk_args),
        Command::Remote(cmd) => match UnixStream::connect("/tmp/test.sock") {
            Ok(stream) => {
                let mut stream = UnixStreamWrapper::new(stream);
                if let Err(e) = stream.write(&cmd) {
                    println!("failed to write {e:?}");
                } else {
                    match stream.read::<()>() {
                        Ok(_) => println!("Received"),
                        Err(_) => println!("Failed to receive response"),
                    }
                }
            }
            Err(e) => println!("failed to connect {e:?}"),
        },
    }
}

fn daemon(gtk_args: &Vec<String>) {
    let app = gtk::Application::builder().application_id(ID).build();

    if let Err(error) = app.register(gio::Cancellable::NONE) {
        gtk::glib::g_error!(LOG_NAME, "Failed to register gtk application {error:?}");
    }

    let (ping_tx, ping_rx) = mpsc::channel();
    let (pong_tx, pong_rx) = mpsc::channel();

    ping_tx.send(app.hold()).expect("Daemon could not ping!");

    app.connect_activate(move |app| {
        if let Ok(app_guard) = ping_rx.try_recv() {
            gtk::glib::g_message!(LOG_NAME, "Daemon has started");

            drop(std::fs::remove_file("/tmp/test.sock"));

            let socket = match UnixListener::bind("/tmp/test.sock") {
                Ok(socket) => socket,
                Err(e) => {
                    eprintln!("failed to connect {e:?}");
                    return;
                }
            };

            pong_tx.send(app_guard).expect("Daemon could not pong!");

            loop {
                match socket.accept() {
                    Err(e) => println!("failed to connect {e:?}"),
                    Ok((stream, _addr)) => {
                        let mut stream = UnixStreamWrapper::new(stream);

                        match stream.read() {
                            Err(e) => println!("failed {e:?}"),
                            Ok(cmd) => {
                                println!("received command: {cmd:?}");
                                match cmd {
                                    RemoteCommand::Quit => {
                                        app.quit();
                                        drop(stream.write(()));
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    app.run_with_args(gtk_args);

    drop(pong_rx.try_recv());
}
