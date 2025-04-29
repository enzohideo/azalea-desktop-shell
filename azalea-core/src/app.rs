use std::{cell::RefCell, rc::Rc, sync::mpsc};

use clap::Parser;
use gtk::{
    gio::{
        self,
        prelude::{ApplicationExt, ApplicationExtManual},
    },
    glib,
    prelude::{GtkApplicationExt, GtkWindowExt},
};

use crate::{
    cli::{DaemonCommand, WindowCommand},
    config::Config,
    socket::{self, r#async::UnixStreamWrapper},
};

use super::cli::{Arguments, Command};

static SOCKET_NAME: &str = "azalea.sock";
static LOG_NAME: &str = "AzaleaDesktopShell";
static ID: &str = "usp.ime.AzaleaDesktopShell";

pub trait Application<InitWrapper>
where
    InitWrapper: clap::Subcommand
        + serde::Serialize
        + serde::de::DeserializeOwned
        + std::fmt::Debug
        + 'static,
    Self: 'static + Sized,
{
    fn run(self, config: Option<Config<InitWrapper>>) {
        let args = Arguments::parse();
        let mut gtk_args = vec![std::env::args().next().unwrap()];
        gtk_args.extend(args.gtk_options.clone());

        // TODO: Parse config from file
        // TODO: Get config filename from cli args
        // TODO: Receive app name, so it can look into ~/.config/appname/settings.json
        // TODO: Generate json schema

        let socket_path = format!("{}/{}", env!("XDG_RUNTIME_DIR"), SOCKET_NAME);

        // TODO: Check if it's remote through dbus
        match args.command {
            Command::Daemon(DaemonCommand::Start) => self.daemon(&gtk_args, socket_path, config),
            cmd => self.remote(cmd, socket_path),
        }
    }

    fn daemon(
        self,
        gtk_args: &Vec<String>,
        socket_path: String,
        config: Option<Config<InitWrapper>>,
    ) {
        let app = gtk::Application::builder().application_id(ID).build();

        if let Err(error) = app.register(gio::Cancellable::NONE) {
            gtk::glib::g_error!(LOG_NAME, "Failed to register gtk application {error:?}");
        }

        let (ping_tx, ping_rx) = mpsc::channel();
        let (pong_tx, pong_rx) = mpsc::channel();

        ping_tx.send(app.hold()).expect("Daemon could not ping!");

        let state = Rc::new(RefCell::new(self));

        app.connect_activate(move |app| {
            if let Ok(app_guard) = ping_rx.try_recv() {
                gtk::glib::g_message!(LOG_NAME, "Daemon has started");

                pong_tx.send(app_guard).expect("Daemon could not pong!");

                if let Some(config) = &config {
                    for dto in &config.windows {
                        // TODO: Take ownership instead of borrow
                        state.borrow_mut().create_window_from_dto(&dto, app)
                    }
                }

                match socket::r#async::UnixListenerWrapper::bind(&socket_path) {
                    Ok(listener) => {
                        glib::spawn_future_local(glib::clone!(
                            #[weak]
                            app,
                            #[weak]
                            state,
                            async move {
                                listener
                                    .loop_accept(async |mut stream: UnixStreamWrapper| {
                                        match stream.read().await {
                                            Ok(cmd) => state.borrow_mut().handle_command(cmd, &app),
                                            Err(e) => {
                                                println!("Failed to read command {e:?}");
                                                return false;
                                            }
                                        };
                                        return true;
                                    })
                                    .await;
                            }
                        ));
                    }
                    Err(e) => println!("Failed to bind unix socket {e:?}"),
                }
            }
        });

        app.run_with_args(gtk_args);

        drop(pong_rx.try_recv());
    }

    fn remote(self, command: Command<InitWrapper>, socket_path: String) {
        match socket::sync::UnixStreamWrapper::connect(socket_path) {
            Ok(mut stream) => {
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

    fn handle_command(&mut self, cmd: Command<InitWrapper>, app: &gtk::Application) {
        match cmd {
            Command::Daemon(DaemonCommand::Start) => {
                // TODO: Warning message;
                todo!()
            }
            Command::Daemon(DaemonCommand::Stop) => {
                app.quit();
            }
            Command::Window(WindowCommand::Create(model)) => {
                self.create_window_from_dto(&model, app)
            }
        }
    }

    fn create_window_from_dto(
        &mut self,
        dto: &crate::model::window::InitDTO<InitWrapper>,
        app: &gtk::Application,
    ) {
        let window = self.create_window(&dto.init);
        window.set_title(Some(&dto.id));
        app.add_window(&window);
        window.present();
    }

    fn create_window(&mut self, init: &InitWrapper) -> gtk::Window;
}
