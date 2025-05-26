use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc, sync::mpsc};

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
    cli,
    config::{self, Config},
    dbus, error, log,
    socket::{self, r#async::UnixStreamWrapper},
};

use super::cli::{Arguments, Command};

pub struct Application<WM, ConfigWrapper, WindowWrapper>
where
    WM: WindowManager<ConfigWrapper, WindowWrapper>,
    ConfigWrapper: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + 'static,
    Self: 'static + Sized,
{
    config: config::Config<ConfigWrapper>,
    dbus: Option<dbus::DBusWrapper>,
    window_manager: WM,
    windows: HashMap<config::window::Id, WindowWrapper>,
}

impl<WM, ConfigWrapper, WindowWrapper> Application<WM, ConfigWrapper, WindowWrapper>
where
    WM: WindowManager<ConfigWrapper, WindowWrapper>,
    ConfigWrapper: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + 'static,
{
    pub fn new(window_manager: WM, config: config::Config<ConfigWrapper>) -> Self {
        Self {
            config,
            dbus: dbus::DBusWrapper::new().ok(),
            window_manager,
            windows: Default::default(),
        }
    }

    fn load_config(path: &PathBuf) -> Result<Config<ConfigWrapper>, error::ConfigError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let ext = path
            .extension()
            .ok_or(error::ConfigError::MissingExtension)?;

        Ok(if ext == "json" {
            serde_json::from_reader(reader)
                .map_err(|e| error::ConfigError::ParsingError(e.to_string()))?
        } else {
            ron::de::from_reader(reader)
                .map_err(|e| error::ConfigError::ParsingError(e.to_string()))?
        })
    }

    pub fn run(self) {
        let args = {
            let arg_style = clap::builder::styling::Style::new().bold().underline();

            Arguments::parse(format!(
                "{}Window IDs:{} {}",
                arg_style.render(),
                arg_style.render_reset(),
                self.config
                    .windows
                    .keys()
                    .fold(format!(""), |acc, v| format!("{acc}\n  {v}"))
            ))
        };

        let socket_path = glib::user_runtime_dir().join(WM::SOCKET_NAME);

        if let Some(dbus) = &self.dbus {
            if dbus.name_has_owner(WM::APP_ID).unwrap_or(false) {
                self.remote(args, socket_path, Some(std::time::Duration::from_secs(1)));
            } else if args.wait_for_daemon {
                log::message!("Waiting for daemon to start");
                drop(dbus.wait_for_name_owner(WM::APP_ID));
                std::thread::sleep(std::time::Duration::from_secs(1));
                self.remote(args, socket_path, Some(std::time::Duration::from_secs(1)));
            } else {
                self.daemon(args, socket_path);
            }
        }
    }

    fn daemon(mut self, args: Arguments, socket_path: PathBuf) {
        match args.command {
            Command::Daemon(cli::daemon::Command::Start {
                config: config_path,
            }) => {
                let config_path = config_path
                    .map(|p| PathBuf::from(&p))
                    .unwrap_or(gtk::glib::user_config_dir().join(WM::CONFIG_PATH));

                match Self::load_config(&config_path) {
                    Ok(config) => {
                        log::message!("Config loaded from {:?}", config_path);
                        self.config = config;
                    }
                    Err(err) => match err {
                        error::ConfigError::Io(_) => {
                            log::message!(
                                "Config not found at {:?}, using default config",
                                config_path
                            )
                        }
                        error => log::warning!(
                            "Config could not be loaded from {:?}, using default config.\n{:?}",
                            config_path,
                            error
                        ),
                    },
                }
            }
            Command::Config(cli::config::Command::View { json }) => {
                println!(
                    "{}",
                    if json {
                        serde_json::to_string_pretty(&self.config).unwrap()
                    } else {
                        use ron::extensions::Extensions;
                        ron::ser::to_string_pretty(
                            &self.config,
                            ron::ser::PrettyConfig::default().extensions(
                                Extensions::IMPLICIT_SOME | Extensions::UNWRAP_VARIANT_NEWTYPES,
                            ),
                        )
                        .unwrap()
                    },
                );
                return;
            }
            _ => {
                log::error!("Daemon isn't running, invalid command: {:?}", args.command);
                return;
            }
        }

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
                log::message!("Daemon started");

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

        {
            let mut gtk_args = vec![std::env::args().next().unwrap()];
            gtk_args.extend(args.gtk_options.clone());
            app.run_with_args(&gtk_args);
        }

        drop(pong_rx.try_recv());
    }

    fn remote(self, args: Arguments, socket_path: PathBuf, retry: Option<std::time::Duration>) {
        loop {
            match socket::sync::UnixStreamWrapper::connect(&socket_path) {
                Ok(mut stream) => {
                    if let Err(e) = stream.write(&args.command) {
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
                    return;
                }
                Err(e) => {
                    if let Some(duration) = retry {
                        std::thread::sleep(duration);
                    } else {
                        log::warning!("failed to connect {e:?}");
                        return;
                    }
                }
            }
        }
    }

    fn handle_command(&mut self, cmd: Command, app: &gtk::Application) {
        match cmd {
            Command::Daemon(cli::daemon::Command::Start { config: _ }) => {
                log::warning!("There's already an instance running")
            }
            Command::Daemon(cli::daemon::Command::Stop) => app.quit(),
            Command::Window(cli::window::Command::Create(arg)) => self.create_window(&arg.id, app),
            Command::Window(cli::window::Command::Toggle(arg)) => {
                let Some(wrapper) = self.windows.get(&arg.id) else {
                    return;
                };
                let window = WM::unwrap_window(wrapper);
                window.set_visible(!window.get_visible());
            }
            Command::Layer(cli::layer_shell::Command::Toggle(arg)) => self
                .windows
                .values()
                .map(|win| WM::unwrap_window(win))
                .filter(|win| arg.cmp(win))
                .for_each(|win| win.set_visible(!win.get_visible())),
            Command::Config(cli::config::Command::View { json: _ }) => todo!(),
        }
    }

    fn create_window(&mut self, id: &config::window::Id, app: &gtk::Application) {
        let Some(window_cfg) = self.config.windows.get(id) else {
            log::warning!("Window configuration not found for id {}", id);
            return;
        };

        if let Some(_) = self.windows.get(id) {
            log::warning!("Window already exists with id {}", id);
            return;
        }

        let wrapped_window = self.window_manager.create_window(&window_cfg.config);
        let window = WM::unwrap_window(&wrapped_window);

        window.set_title(Some(&id));

        if let Some(layer_shell) = &window_cfg.layer_shell {
            window.init_layer_shell();
            window.set_namespace(Some(&layer_shell.namespace));
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
    ConfigWrapper: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + 'static,
    Self: 'static + Sized,
{
    const CONFIG_PATH: &str = "azalea/config.ron";
    const SOCKET_NAME: &str = "azalea.sock";
    const APP_ID: &str = "br.usp.ime.Azalea";

    fn create_window(&self, config: &ConfigWrapper) -> WindowWrapper;
    fn unwrap_window(window: &WindowWrapper) -> &gtk::Window;
}
