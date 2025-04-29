use azalea::core::{
    app::{self, Application},
    config::Config,
    model,
};
use azalea::window::taskbar;

// TODO: Macro to create Init based on list of widgets?
#[derive(clap::Subcommand, serde::Serialize, serde::Deserialize, Debug)]
pub enum Init {
    Default,
    Taskbar(taskbar::Init),
}

pub struct AzaleaDesktopShell {}
impl app::Application<Init> for AzaleaDesktopShell {}

fn main() {
    let config = Config {
        windows: vec![model::window::Init {
            id: format!("default"),
            init: Init::Taskbar(taskbar::Init {}),
            layer_shell: None,
        }],
    };
    AzaleaDesktopShell::run(Some(config));
}
