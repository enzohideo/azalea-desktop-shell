use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::mpsc};

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

pub struct Application<WM, ConfigWrapper, WindowWrapper>
where
    WM: WindowManager<ConfigWrapper, WindowWrapper>,
    ConfigWrapper: clap::Subcommand
        + serde::Serialize
        + serde::de::DeserializeOwned
        + std::fmt::Debug
        + 'static,
    Self: 'static + Sized,
{
    config: config::Config<ConfigWrapper>,
    window_manager: WM,
    windows: HashMap<config::window::Id, WindowWrapper>,
}

impl<WM, ConfigWrapper, WindowWrapper> Application<WM, ConfigWrapper, WindowWrapper>
where
    WM: WindowManager<ConfigWrapper, WindowWrapper>,
    ConfigWrapper: clap::Subcommand
        + serde::Serialize
        + serde::de::DeserializeOwned
        + std::fmt::Debug
        + 'static,
{
    pub fn new(window_manager: WM, config: config::Config<ConfigWrapper>) -> Self {
        Self {
            config,
            window_manager,
            windows: Default::default(),
        }
    }

    fn load_config(path: &std::path::PathBuf) -> Option<Config<ConfigWrapper>> {
        let file = std::fs::File::open(path).ok()?;
        let reader = std::io::BufReader::new(file);

        match serde_json::from_reader(reader) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                log::warning!("Failed to parse config from file {path:?}\n{e}");
                None
            }
        }
    }

    pub fn run(mut self) {
        let args = Arguments::parse();
        let mut gtk_args = vec![std::env::args().next().unwrap()];
        gtk_args.extend(args.gtk_options.clone());

        let socket_path = format!("{}/{}", env!("XDG_RUNTIME_DIR"), WM::SOCKET_NAME);

        // TODO: Check if it's remote through dbus
        match args.command {
            Command::Daemon(DaemonCommand::Start {
                config: config_path,
            }) => {
                let config_path = config_path
                    .map(|p| std::path::PathBuf::from(&p))
                    .unwrap_or(gtk::glib::user_config_dir().join(WM::CONFIG_PATH));

                if let Some(config) = Self::load_config(&config_path) {
                    log::message!("Loaded config from file {:?}", config_path);
                    self.config = config;
                }

                self.daemon(&gtk_args, socket_path)
            }
            cmd => self.remote(cmd, socket_path),
        }
    }

    fn daemon(self, gtk_args: &Vec<String>, socket_path: String) {
        let app = gtk::Application::builder()
            .application_id(WM::APP_ID)
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

                {
                    let config_window_ids: Vec<config::window::Id> = state
                        .borrow()
                        .config
                        .windows
                        .keys()
                        .map(|v| v.to_owned())
                        .collect();
                    let mut state = state.borrow_mut();
                    for id in config_window_ids {
                        let Some(window_cfg) = state.config.windows.get(&id) else {
                            continue;
                        };

                        if window_cfg.lazy {
                            continue;
                        }

                        state.create_window(&id, app)
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

    fn remote(self, command: Command, socket_path: String) {
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

    fn handle_command(&mut self, cmd: Command, app: &gtk::Application) {
        match cmd {
            Command::Daemon(DaemonCommand::Start { config: _ }) => {
                log::warning!("There's already an instance running")
            }
            Command::Daemon(DaemonCommand::Stop) => app.quit(),
            Command::Window(WindowCommand::Create(header)) => self.create_window(&header.id, app),
            Command::Window(WindowCommand::Toggle(header)) => {
                let Some(wrapper) = self.windows.get(&header.id) else {
                    return;
                };
                let window = WM::unwrap_window(wrapper);
                window.set_visible(!window.get_visible());
            }
        }
    }

    fn create_window(&mut self, id: &config::window::Id, app: &gtk::Application) {
        let Some(window_cfg) = self.config.windows.get(id) else {
            log::warning!("Window configuration not found for id {}", id);
            return;
        };
        // TODO: Check if window exists
        let wrapped_window = self.window_manager.create_window(&window_cfg.config);
        let window = WM::unwrap_window(&wrapped_window);

        window.set_title(Some(&id));

        if let Some(layer_shell) = &window_cfg.layer_shell {
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

        self.windows.insert(id.clone(), wrapped_window);
    }
}

pub trait WindowManager<ConfigWrapper, WindowWrapper>
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

    fn create_window(&self, config: &ConfigWrapper) -> WindowWrapper;
    fn unwrap_window(window: &WindowWrapper) -> &gtk::Window;
}
