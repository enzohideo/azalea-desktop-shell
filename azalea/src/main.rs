use std::collections::HashMap;

use azalea::{
    core::{
        app::{self, Application},
        config,
    },
    window::{self, taskbar},
};
use relm4::{Component, ComponentController};

// TODO: Macro to create Init based on list of widgets?
#[derive(clap::Subcommand, serde::Serialize, serde::Deserialize, Debug)]
pub enum ConfigWrapper {
    Default,
    Taskbar(taskbar::Config),
}

pub enum WindowWrapper {
    Default(gtk::Window),
    Taskbar(relm4::component::Controller<taskbar::Model>),
}

pub struct AzaleaDesktopShell {
    windows: HashMap<config::window::Id, WindowWrapper>,
}

impl app::Application<ConfigWrapper, WindowWrapper> for AzaleaDesktopShell {
    fn create_window(&self, init: &ConfigWrapper) -> WindowWrapper {
        match &init {
            ConfigWrapper::Default => {
                let btn = gtk::Button::with_label("Hey");
                let window = gtk::Window::builder().child(&btn).build();
                WindowWrapper::Default(window)
            }
            ConfigWrapper::Taskbar(config) => {
                let builder = taskbar::Model::builder();
                let controller = builder
                    .launch(window::Init::<taskbar::Config> {
                        config: config.clone(),
                    })
                    .detach();
                WindowWrapper::Taskbar(controller)
            }
        }
    }

    fn store_window(&mut self, id: config::window::Id, window: WindowWrapper) {
        self.windows.insert(id, window);
    }

    fn unwrap_window(window: &WindowWrapper) -> &gtk::Window {
        match window {
            WindowWrapper::Default(window) => window,
            WindowWrapper::Taskbar(controller) => controller.widget(),
        }
    }
}

fn main() {
    let config = config::Config {
        windows: vec![config::window::Config {
            id: format!("bottom-taskbar"),

            config: ConfigWrapper::Taskbar({
                use taskbar::{Config, widget::Kind::*};

                Config {
                    start: vec![],
                    center: vec![],
                    end: vec![Time],
                }
            }),

            layer_shell: Some({
                use config::layer_shell::{Anchor, Config, Layer};

                Config {
                    namespace: Some(format!("taskbar")),
                    layer: Layer::Bottom,
                    anchors: vec![Anchor::Left, Anchor::Right, Anchor::Bottom],
                    auto_exclusive_zone: true,
                }
            }),
        }],
    };

    let app = AzaleaDesktopShell {
        windows: Default::default(),
    };

    app.run(Some(config));
}
