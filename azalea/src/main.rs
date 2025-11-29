use std::collections::HashMap;

use azalea::{
    core::{
        app::{self},
        config,
    },
    shell::{self, icon, window::taskbar},
};
use azalea_core::{config::Config, monitor::Monitor};
use azalea_shell::window::wallpaper;
use relm4::{Component, ComponentController};

// TODO: Macro to create Init based on list of widgets?
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ConfigWrapper {
    Default,
    Taskbar(taskbar::Config),
    Wallpaper(wallpaper::Config),
}

pub enum WindowWrapper {
    Default(gtk::Window),
    Taskbar(relm4::component::Controller<taskbar::Model>),
    Wallpaper(relm4::component::Controller<wallpaper::Model>),
}

pub struct AzaleaAppExt {}

impl app::AzaleaAppExt for AzaleaAppExt {
    type ConfigWrapper = ConfigWrapper;
    type WindowWrapper = WindowWrapper;

    fn create_window(init: &ConfigWrapper) -> WindowWrapper {
        match &init {
            ConfigWrapper::Default => {
                let btn = gtk::Button::with_label("Hey");
                let window = gtk::Window::builder().child(&btn).build();
                WindowWrapper::Default(window)
            }
            ConfigWrapper::Taskbar(config) => {
                let builder = taskbar::Model::builder();
                let controller = builder
                    .launch(shell::window::Init::<taskbar::Model> {
                        config: config.clone(),
                    })
                    .detach();
                WindowWrapper::Taskbar(controller)
            }
            ConfigWrapper::Wallpaper(config) => {
                let builder = wallpaper::Model::builder();
                let controller = builder
                    .launch(shell::window::Init::<wallpaper::Model> {
                        config: config.clone(),
                    })
                    .detach();
                WindowWrapper::Wallpaper(controller)
            }
        }
    }

    fn unwrap_window(window: &WindowWrapper) -> &gtk::Window {
        match window {
            WindowWrapper::Default(window) => window,
            WindowWrapper::Taskbar(controller) => controller.widget(),
            WindowWrapper::Wallpaper(controller) => controller.widget(),
        }
    }
}

fn main() {
    icon::init();

    let windows = HashMap::from([(
        format!("bottom-taskbar"),
        config::window::Config {
            config: ConfigWrapper::Taskbar({
                use taskbar::{
                    Config,
                    widget::{ConfigWrapper::*, bluetooth, media, network, search, time},
                };

                Config {
                    spacing: 8,

                    start: vec![Search(search::Config { top_down: false })],

                    center: vec![Media(media::Config {})],

                    end: vec![
                        Bluetooth(bluetooth::Config {}),
                        Network(network::Config {}),
                        Time(time::Config {
                            format: format!("%d/%m/%y"),
                        }),
                        Time(time::Config {
                            format: format!("%H:%M:%S"),
                        }),
                    ],
                }
            }),

            layer_shell: Some({
                use config::layer_shell::{Anchor, Config, Layer};

                Config {
                    namespace: format!("taskbar"),
                    layer: Layer::Top,
                    anchors: vec![Anchor::Left, Anchor::Right, Anchor::Bottom],
                    auto_exclusive_zone: true,
                }
            }),

            lazy: false,

            monitor: Monitor::All,
        },
    )]);

    app::AzaleaApp::<AzaleaAppExt>::new(Config { windows }).run();
}
