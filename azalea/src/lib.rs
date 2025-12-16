#![doc(html_logo_url = "../azalea-logo.png")]

//! # azalea
//!
//! This crate works as the main application and as a library that re-exports the inner crates
//!
//! Using this as a library:
//!```bash
//!cargo add --git https://github.com/enzohideo/azalea-desktop-shell.git azalea
//!```
//!
//!You need to crate two wrappers: `ConfigWrapper` and `WindowWrapper`.
//!
//!The `ConfigWrapper` wraps each window's configuration
//!```rust
//!#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
//!pub enum ConfigWrapper {
//!    Default,
//!    Taskbar(taskbar::Config),
//!}
//!```
//!
//!The `WindowWrapper` wraps data structures that contains a gtk::Window
//!```rust
//!pub enum WindowWrapper {
//!    Default(gtk::Window),
//!    Taskbar(relm4::component::Controller<taskbar::Model>),
//!}
//!```
//!
//!Now, you just need to implement [core::app::AzaleaAppExt]
//!```
//!pub struct AzaleaAppExt {}
//!
//!impl app::AzaleaAppExt for AzaleaAppExt {
//!
//!    type ConfigWrapper = ConfigWrapper;
//!    type WindowWrapper = WindowWrapper;
//!
//!    fn create_window(init: &ConfigWrapper) -> WindowWrapper {
//!        match &init {
//!            ConfigWrapper::Default => {
//!                let btn = gtk::Button::with_label("Hey");
//!                let window = gtk::Window::builder().child(&btn).build();
//!                WindowWrapper::Default(window)
//!            }
//!            ConfigWrapper::Taskbar(config) => {
//!                let builder = taskbar::Model::builder();
//!                let controller = builder
//!                    .launch(shell::window::Init::<taskbar::Model> {
//!                        config: config.clone(),
//!                    })
//!                    .detach();
//!                WindowWrapper::Taskbar(controller)
//!            }
//!        }
//!    }
//!
//!    fn unwrap_window(window: &WindowWrapper) -> &gtk::Window {
//!        match window {
//!            WindowWrapper::Default(window) => window,
//!            WindowWrapper::Taskbar(controller) => controller.widget(),
//!        }
//!    }
//!}
//!```
//!
//!Now, the main application
//!```rust
//!fn main() {
//!    // Initialize the icons
//!    icon::init();
//!
//!    // Default windows' configurations
//!    let windows = HashMap::from([
//!        (
//!            format!("hey"),
//!            config::window::Config {
//!                config: ConfigWrapper::Default,
//!
//!                layer_shell: Some({
//!                    use config::layer_shell::{Anchor, Config, ExclusiveZone, Layer};
//!
//!                    Config {
//!                        namespace: format!("something"),
//!                        layer: Layer::Background,
//!                        anchors: vec![Anchor::Left, Anchor::Right, Anchor::Bottom, Anchor::Top],
//!                        exclusive_zone: ExclusiveZone::Ignore,
//!                    }
//!                }),
//!
//!                lazy: false,
//!
//!                monitor: Monitor::All,
//!            },
//!        ),
//!        (
//!            format!("bottom-taskbar"),
//!            config::window::Config {
//!                config: ConfigWrapper::Taskbar({
//!                    use taskbar::{
//!                        Config,
//!                        widget::{ConfigWrapper::*, *},
//!                    };
//!
//!                    Config {
//!                        spacing: 8,
//!
//!                        start: vec![
//!                            StartMenu(startmenu::Config {}),
//!                            Separator(separator::Config { separator: None }),
//!                            Audio(audio::Config {}),
//!                            Separator(separator::Config { separator: None }),
//!                            Brightness(brightness::Config {}),
//!                            Separator(separator::Config { separator: None }),
//!                        ],
//!
//!                        center: vec![Media(media::Config {})],
//!
//!                        end: vec![
//!                            Separator(separator::Config { separator: None }),
//!                            Bluetooth(bluetooth::Config {}),
//!                            Separator(separator::Config { separator: None }),
//!                            Network(network::Config {}),
//!                            Separator(separator::Config { separator: None }),
//!                            Time(time::Config {
//!                                format: format!("%d/%m/%y"),
//!                            }),
//!                            Separator(separator::Config { separator: None }),
//!                            Time(time::Config {
//!                                format: format!("%H:%M"),
//!                            }),
//!                            Separator(separator::Config { separator: None }),
//!                        ],
//!                    }
//!                }),
//!
//!                layer_shell: Some({
//!                    use config::layer_shell::{Anchor, Config, ExclusiveZone, Layer};
//!
//!                    Config {
//!                        namespace: format!("taskbar"),
//!                        layer: Layer::Top,
//!                        anchors: vec![Anchor::Left, Anchor::Right, Anchor::Bottom],
//!                        exclusive_zone: ExclusiveZone::Auto,
//!                    }
//!                }),
//!
//!                lazy: false,
//!
//!                monitor: Monitor::All,
//!            },
//!        ),
//!    ]);
//!
//!    app::AzaleaApp::<AzaleaAppExt>::new(Config { windows }).run();
//!}
//!```
//!
//!If you want to use a configuration file, use `azalea config view` to see the default
//!configuration in RON format, then copy it to `~/.config/azalea/config.ron`
//!
//!You can also change where it looks for config files by overriding it in [core::app::AzaleaAppExt]
//!```rust
//!impl app::AzaleaAppExt for AzaleaAppExt {
//!   const CONFIG_PATH: &str = "azalea/config.ron";
//!
//!   // You can also use JSON, instead of RON
//!   // const CONFIG_PATH: &str = "azalea/config.json";
//!
//!   // Other things you can also override:
//!   const STYLE_PATH: &str = "azalea/style.scss";
//!   const SOCKET_NAME: &str = "azalea.sock";
//!   const APP_ID: &str = "br.usp.ime.Azalea";
//!}
//!```

#[doc(inline)]
pub use azalea_core as core;

#[doc(inline)]
pub use azalea_log as log;

#[doc(inline)]
pub use azalea_derive as derive;

#[doc(inline)]
pub use azalea_shell as shell;

#[doc(inline)]
pub use azalea_service as service;
