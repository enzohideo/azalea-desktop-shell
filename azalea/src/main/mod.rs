use azalea::{
    app::{self, Application},
    config::Config,
    model,
};

// TODO: Macro to create Init based on list of widgets?
#[derive(clap::Subcommand, bincode::Encode, bincode::Decode, Debug)]
pub enum Init {
    Default,
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
