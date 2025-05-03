use std::collections::HashMap;

use azalea::core::{
    app::{self, Application},
    config::Config,
    model,
};
use azalea::window::taskbar;
use relm4::{Component, ComponentController};

// TODO: Macro to create Init based on list of widgets?
#[derive(clap::Subcommand, serde::Serialize, serde::Deserialize, Debug)]
pub enum InitWrapper {
    Default,
    Taskbar(taskbar::Init),
}

pub enum WindowWrapper {
    Default(gtk::Window),
    Taskbar(relm4::component::Controller<taskbar::Model>),
}

pub struct AzaleaDesktopShell {
    windows: HashMap<model::window::Id, WindowWrapper>,
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

    fn store_window(&mut self, id: model::window::Id, window: WindowWrapper) {
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
    let config = Config {
        windows: vec![model::window::InitDTO {
            id: format!("bottom-taskbar"),
            init: InitWrapper::Taskbar({
                use taskbar::widget::Kind::*;

                taskbar::Init {
                    start: vec![],
                    center: vec![],
                    end: vec![Time],
                }
            }),
            layer_shell: Some(model::layer_shell::Model {
                namespace: Some(format!("taskbar")),
                layer: Some(model::layer_shell::Layer::Bottom),
                anchors: vec![
                    model::layer_shell::Anchor::Left,
                    model::layer_shell::Anchor::Right,
                    model::layer_shell::Anchor::Bottom,
                ],
                auto_exclusive_zone: true,
            }),
        }],
    };

    let app = AzaleaDesktopShell {
        windows: Default::default(),
    };

    app.run(Some(config));
}
