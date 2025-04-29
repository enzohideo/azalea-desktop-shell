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

pub enum ControllerWrapper {
    Default,
    Taskbar(relm4::component::Controller<taskbar::Model>),
}

#[derive(Default)]
pub struct AzaleaDesktopShell {
    connectors: Vec<ControllerWrapper>,
}

impl app::Application<InitWrapper> for AzaleaDesktopShell {
    fn create_window(&mut self, init: &InitWrapper) -> gtk::Window {
        match &init {
            InitWrapper::Default => {
                let btn = gtk::Button::with_label("Hey");
                let window = gtk::Window::builder().child(&btn).build();
                window
            }
            InitWrapper::Taskbar(init) => {
                let builder = taskbar::Model::builder();
                let controller = builder.launch(init.clone()).detach();
                let window = controller.widget().clone();
                self.connectors.push(ControllerWrapper::Taskbar(controller));
                window
            }
        }
    }
}

fn main() {
    let config = Config {
        windows: vec![model::window::InitData {
            id: format!("default"),
            init: InitWrapper::Taskbar(taskbar::Init {}),
            layer_shell: None,
        }],
    };

    let app = AzaleaDesktopShell {
        ..Default::default()
    };

    app.run(Some(config));
}
