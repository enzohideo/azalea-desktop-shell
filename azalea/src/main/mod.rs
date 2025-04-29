// TODO: Separate azalea lib and azalea-core (re-export it in azalea)
use azalea::{
    app::{self, Application},
    config::Config,
    model,
};

// TODO: Macro to create Init based on list of widgets?
#[derive(clap::Subcommand, bincode::Encode, bincode::Decode, Debug)]
pub enum Init {
    Default,
    // TODO: Add this after change bincode -> serde.
    // Taskbar(window::taskbar::Init)
}

pub struct AzaleaDesktopShell {}
impl app::Application<Init> for AzaleaDesktopShell {}

fn main() {
    let config = Config {
        windows: vec![model::window::Init {
            id: format!("default"),
            init: Init::Default,
            layer_shell: None,
        }],
    };
    AzaleaDesktopShell::run(Some(config));
}
