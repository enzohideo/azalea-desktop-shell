use std::{collections::HashMap, rc::Rc};

use azalea::{
    core::{
        app::{self},
        config,
    },
    service::{self, IntoServices, Service},
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

azalea_service::services! {
    require time: azalea_service::time::Model;
}

pub struct WindowManager {
    services: Services,
}

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
                        services: self.services.strip(),
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
    let time_service = Service::new(std::time::Duration::from_millis(1000));
    drop(time_service.send(service::Input::Start));

    relm4::view!(
        mut windows = HashMap::new() {
            insert: (format!("bottom-taskbar"), config::window::Config {
                config: ConfigWrapper::Taskbar({
                    use taskbar::{Config, widget::Kind::*};

                    Config {
                        start: vec![],
                        center: vec![],
                        end: vec![Time(taskbar::widget::time::Config {})],
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

                lazy: false,
            }),
        }
    );

    app::Application::new(
        WindowManager {
            services: Services {
                time: Rc::new(time_service),
            },
        },
        Config { windows },
    )
    .run();
}
