use std::collections::HashMap;

use azalea::core::{
    app::{self, Application},
    config,
};
use azalea::window::taskbar;
use relm4::{Component, ComponentController};

// TODO: Macro to create Init based on list of widgets?
#[derive(clap::Subcommand, serde::Serialize, serde::Deserialize, Debug)]
pub enum InitWrapper {
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

impl app::Application<InitWrapper, WindowWrapper> for AzaleaDesktopShell {
    fn create_window(&self, init: &InitWrapper) -> WindowWrapper {
        match &init {
            InitWrapper::Default => {
                let btn = gtk::Button::with_label("Hey");
                let window = gtk::Window::builder().child(&btn).build();
                WindowWrapper::Default(window)
            }
            InitWrapper::Taskbar(init) => {
                let builder = taskbar::Model::builder();
                let controller = builder.launch(init.clone()).detach();
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

            init: InitWrapper::Taskbar({
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
                    layer: Some(Layer::Bottom),
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
