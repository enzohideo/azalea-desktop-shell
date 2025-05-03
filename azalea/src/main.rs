use std::collections::HashMap;

use azalea::window::taskbar;
use azalea::{
    core::{
        app::{self, Application},
        config::Config,
        model,
    },
    service,
};
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
    time_service: service::Service<service::time::Model>,
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
                let sender = controller.sender().clone();

                self.time_service
                    .forward(sender, taskbar::Input::UpdateTime);

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
            init: InitWrapper::Taskbar(taskbar::Init {}),
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

    let time_service = service::time::Model::builder()
        .detach_worker(std::time::Duration::from_secs(1))
        .into();

    let app = AzaleaDesktopShell {
        windows: Default::default(),
        time_service,
    };

    app.run(Some(config));
}
