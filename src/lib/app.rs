use std::{os::unix::net::UnixStream, sync::mpsc};

use clap::Parser;
use gtk::{
    gio::{
        self,
        prelude::{ApplicationExt, ApplicationExtManual},
    },
    prelude::{GtkApplicationExt, GtkWindowExt},
};

use crate::{
    cli::RemoteCommand,
    socket::{UnixListenerWrapper, UnixStreamWrapper},
};

use super::cli::{Arguments, Command};

static SOCKET_NAME: &str = "lily.sock";
static LOG_NAME: &str = "LilyDesktopShell";
static ID: &str = "usp.ime.LilyDesktopShell";

pub fn run() {
    let args = Arguments::parse();
    let mut gtk_args = vec![std::env::args().next().unwrap()];
    gtk_args.extend(args.gtk_options.clone());

    let socket_path = format!("{}/{}", env!("XDG_RUNTIME_DIR"), SOCKET_NAME);

    // TODO: Check if it's remote through dbus
    match args.command {
        Command::Daemon => daemon(&gtk_args, socket_path),
        Command::Remote(cmd) => remote(cmd, socket_path),
    }
}

fn daemon(gtk_args: &Vec<String>, socket_path: String) {
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

            pong_tx.send(app_guard).expect("Daemon could not pong!");
            if let Err(e) = UnixListenerWrapper::bind(&socket_path).and_then(|listener| {
                listener.loop_accept(|mut stream| {
                    let cmd = stream.read()?;
                    println!("received command: {cmd:?}");

                    match cmd {
                        RemoteCommand::Quit => {
                            app.quit();
                            drop(stream.write(()));
                            return Ok(false);
                        }
                        RemoteCommand::Create => {
                            // FIXME: Some how make unix sockets work without blocking the main
                            // loop or use actions (or dbus directly)
                            let btn = gtk::Button::with_label("Hey");
                            let window = gtk::Window::builder()
                                .application(app)
                                .title("Hello World")
                                .child(&btn)
                                .build();
                            window.present();
                        }
                    }

                    Ok(true)
                })
            }) {
                println!("Failed to bind unix socket {e:?}");
            };
        }
    });

    app.run_with_args(gtk_args);

    drop(pong_rx.try_recv());
}

fn remote(command: RemoteCommand, socket_path: String) {
    match UnixStream::connect(socket_path) {
        Ok(stream) => {
            let mut stream = UnixStreamWrapper::new(stream);
            if let Err(e) = stream.write(&command) {
                println!("failed to write {e:?}");
            } else {
                match stream.read::<()>() {
                    Ok(_) => println!("Received"),
                    Err(_) => println!("Failed to receive response"),
                }
            }
        }
        Err(e) => println!("failed to connect {e:?}"),
    }
}
