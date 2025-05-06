use std::{cell::RefCell, rc::Rc, sync::mpsc};

use clap::Parser;
use gtk::{
    gio::{
        self,
        prelude::{ApplicationExt, ApplicationExtManual},
    },
    glib,
    prelude::{GtkApplicationExt, GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;

use crate::{
    cli::{self, DaemonCommand, WindowCommand},
    config::{self, Config},
    log,
    socket::{self, r#async::UnixStreamWrapper},
};

use super::cli::{Arguments, Command};

pub trait Application<ConfigWrapper, WindowWrapper>
where
    ConfigWrapper: clap::Subcommand
        + serde::Serialize
        + serde::de::DeserializeOwned
        + std::fmt::Debug
        + 'static,
    Self: 'static + Sized,
{
    const CONFIG_PATH: &str = "azalea/config.json";
    const SOCKET_NAME: &str = "azalea.sock";
    const APP_ID: &str = "br.usp.ime.Azalea";

    fn run(self, config: Option<Config<ConfigWrapper>>) {
        let args = Arguments::parse();
        let mut gtk_args = vec![std::env::args().next().unwrap()];
        gtk_args.extend(args.gtk_options.clone());

        // TODO: Parse config from file
        // TODO: Get config filename from cli args
        // TODO: Receive app name, so it can look into ~/.config/appname/settings.json
        // TODO: Generate json schema

        let socket_path = format!("{}/{}", env!("XDG_RUNTIME_DIR"), Self::SOCKET_NAME);

        // TODO: Check if it's remote through dbus
        match args.command {
            Command::Daemon(DaemonCommand::Start {
                config: config_path,
            }) => {
                let config_path = config_path
                    .map(|p| std::path::PathBuf::from(&p))
                    .unwrap_or(gtk::glib::user_config_dir().join(Self::CONFIG_PATH));
                let config_from_file = Self::load_config(&config_path);
                let config = match config_from_file {
                    None => config,
                    config => {
                        log::message!("Loaded config from file {:?}", config_path);
                        config
                    }
                };
                self.daemon(&gtk_args, socket_path, config)
            }
            cmd => self.remote(cmd, socket_path),
        }
    }

    fn load_config(path: &std::path::PathBuf) -> Option<Config<ConfigWrapper>> {
        let file = std::fs::File::open(path).ok()?;
        let reader = std::io::BufReader::new(file);
        Some(serde_json::from_reader(reader).ok()?)
    }

    fn daemon(
        self,
        gtk_args: &Vec<String>,
        socket_path: String,
        config: Option<Config<ConfigWrapper>>,
    ) {
        let app = gtk::Application::builder()
            .application_id(Self::APP_ID)
            .build();

        if let Err(error) = app.register(gio::Cancellable::NONE) {
            log::error!("Failed to register gtk application {error:?}");
        }

        let (ping_tx, ping_rx) = mpsc::channel();
        let (pong_tx, pong_rx) = mpsc::channel();

        ping_tx.send(app.hold()).expect("Daemon could not ping!");

        let state = Rc::new(RefCell::new(self));

        app.connect_activate(move |app| {
            if let Ok(app_guard) = ping_rx.try_recv() {
                log::message!("Daemon has started");

                pong_tx.send(app_guard).expect("Daemon could not pong!");

                if let Some(config) = &config {
                    for dto in &config.windows {
                        // TODO: Take ownership instead of borrow
                        state.borrow_mut().create_layer_shell(&dto, app)
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
                                            Ok(cmd) => {
                                                state.borrow_mut().handle_command(cmd, &app);
                                                // TODO: Allow commands with custom responses
                                                let answer = cli::Response::Success(format!("Ok"));
                                                drop(stream.write(answer).await);
                                                return true;
                                            }
                                            Err(e) => {
                                                let answer = cli::Response::Error(format!("{e:?}"));
                                                drop(stream.write(answer).await);
                                                return false;
                                            }
                                        };
                                    })
                                    .await;
                            }
                        ));
                    }
                    Err(e) => log::error!("Failed to bind unix socket {e:?}"),
                }
            }
        });

        app.run_with_args(gtk_args);

        drop(pong_rx.try_recv());
    }

    fn remote(self, command: Command<ConfigWrapper>, socket_path: String) {
        match socket::sync::UnixStreamWrapper::connect(socket_path) {
            Ok(mut stream) => {
                if let Err(e) = stream.write(&command) {
                    log::warning!("failed to write {e:?}");
                } else {
                    match stream.read::<cli::Response>() {
                        Ok(response) => match response {
                            cli::Response::Success(ans) => println!("{ans}"),
                            cli::Response::Error(e) => log::warning!("{e:?}"),
                        },
                        Err(e) => log::warning!("Failed to receive response: {e:?}"),
                    }
                }
            }
            Err(e) => log::warning!("failed to connect {e:?}"),
        }
    }

    fn handle_command(&mut self, cmd: Command<ConfigWrapper>, app: &gtk::Application) {
        match cmd {
            Command::Daemon(DaemonCommand::Start { config: _ }) => {
                log::warning!("There's already an instance running")
            }
            Command::Daemon(DaemonCommand::Stop) => app.quit(),
            Command::Window(WindowCommand::Create(dto)) => self.create_layer_shell(&dto, app),
            Command::Window(WindowCommand::Toggle(header)) => {
                let Some(wrapper) = self.retrieve_window(header.id) else {
                    return;
                };
                let window = Self::unwrap_window(wrapper);
                window.set_visible(!window.get_visible());
            }
        }
    }

    fn create_layer_shell(
        &mut self,
        dto: &crate::config::window::Config<ConfigWrapper>,
        app: &gtk::Application,
    ) {
        let wrapped_window = self.create_window(&dto.config);
        let window = Self::unwrap_window(&wrapped_window);

        window.set_title(Some(&dto.header.id));

        if let Some(layer_shell) = &dto.layer_shell {
            window.init_layer_shell();
            window.set_namespace(layer_shell.namespace.as_deref());
            window.set_layer((&layer_shell.layer).into());
            for anchor in &layer_shell.anchors {
                window.set_anchor(anchor.into(), true);
            }
            if layer_shell.auto_exclusive_zone {
                window.auto_exclusive_zone_enable();
            }
        }

        app.add_window(window);
        window.present();

        self.store_window(dto.header.id.clone(), wrapped_window);
    }

    fn create_window(&self, config: &ConfigWrapper) -> WindowWrapper;
    fn store_window(&mut self, id: config::window::Id, window: WindowWrapper);
    fn retrieve_window(&mut self, id: config::window::Id) -> Option<&WindowWrapper>;
    fn unwrap_window(window: &WindowWrapper) -> &gtk::Window;
}
