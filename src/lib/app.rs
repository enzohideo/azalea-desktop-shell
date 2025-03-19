use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::mpsc,
};

use clap::Parser;
use gtk::gio::{
    self,
    prelude::{ApplicationExt, ApplicationExtManual},
};

use super::cli::{Arguments, Command, DaemonCommand};

static LOG_NAME: &str = "LilyDesktopShell";
static ID: &str = "usp.ime.LilyDesktopShell";

pub fn run() {
    let args = Arguments::parse();
    let mut gtk_args = vec![std::env::args().next().unwrap()];
    gtk_args.extend(args.gtk_options.clone());

    // TODO: Handle remote commands
    match args.command {
        Command::Daemon(daemon_command) => daemon(daemon_command, &gtk_args),
    }
}

fn daemon(command: DaemonCommand, gtk_args: &Vec<String>) {
    match command {
        DaemonCommand::Start => {
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
                            Ok((mut stream, _addr)) => {
                                let mut response = String::new();
                                if let Err(e) = stream.read_to_string(&mut response) {
                                    println!("failed {e:?}");
                                } else {
                                    // TODO: Handle different remote requests
                                    println!("daemon: {response:?}");
                                    app.quit();
                                    return;
                                }
                            }
                            Err(e) => println!("failed to connect {e:?}"),
                        }
                    }
                }
            });

            app.run_with_args(gtk_args);

            drop(pong_rx.try_recv());
        }
        DaemonCommand::Stop => match UnixStream::connect("/tmp/test.sock") {
            Ok(mut stream) => {
                if let Err(e) = stream.write_all(b"hello world") {
                    println!("failed to write {e:?}");
                }
            }
            Err(e) => println!("failed to connect {e:?}"),
        },
    }
}
