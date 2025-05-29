use std::collections::HashMap;

use azalea::{
    core::{
        app::{self},
        config,
    },
    window::{self, taskbar},
};
use azalea_core::config::Config;
use relm4::{Component, ComponentController};

// TODO: Macro to create Init based on list of widgets?
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ConfigWrapper {
    Default,
    Taskbar(taskbar::Config),
}

pub enum WindowWrapper {
    Default(gtk::Window),
    Taskbar(relm4::component::Controller<taskbar::Model>),
}

pub struct WindowManager {}

impl app::WindowManager<ConfigWrapper, WindowWrapper> for WindowManager {
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
                    .launch(window::Init::<taskbar::Model> {
                        config: config.clone(),
                    })
                    .detach();
                WindowWrapper::Taskbar(controller)
            }
        }
    }

    fn unwrap_window(window: &WindowWrapper) -> &gtk::Window {
        match window {
            WindowWrapper::Default(window) => window,
            WindowWrapper::Taskbar(controller) => controller.widget(),
        }
    }
}

fn main() {
    relm4::view!(
        mut windows = HashMap::new() {
            insert: (format!("bottom-taskbar"), config::window::Config {
                config: ConfigWrapper::Taskbar({
                    use taskbar::{Config, widget::Kind::*};

                    Config {
                        start: vec![],
                        center: vec![],
                        end: vec![
                            Time(taskbar::widget::time::Config {
                                format: format!("%d/%m/%y")
                            }),
                            Time(taskbar::widget::time::Config {
                                format: format!("%H:%M:%S")
                            })
                        ],
                    }
                }),

                layer_shell: Some({
                    use config::layer_shell::{Anchor, Config, Layer};

                    Config {
                        namespace: format!("taskbar"),
                        layer: Layer::Bottom,
                        anchors: vec![Anchor::Left, Anchor::Right, Anchor::Bottom],
                        auto_exclusive_zone: true,
                    }
                }),

                lazy: false,
            }),
        }
    );

    app::Application::new(WindowManager {}, Config { windows }).run();
}
