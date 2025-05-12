use clap::{Parser, arg, command};

use crate::log;

#[derive(clap::Parser, serde::Serialize)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,

    /// Wait for daemon to start
    #[clap(short, long)]
    pub wait_for_daemon: bool,

    /// Arguments passed to GTK.
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub gtk_options: Vec<String>,
}

impl Arguments {
    pub fn parse(after_help: impl clap::builder::IntoResettable<clap::builder::StyledStr>) -> Self {
        let mut matches = <Self as clap::CommandFactory>::command()
            .after_help(after_help)
            .get_matches();
        let res = <Self as clap::FromArgMatches>::from_arg_matches_mut(&mut matches);
        match res {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to parse cli arguments {e}");
                e.exit()
            }
        }
    }
}

#[derive(Parser, serde::Serialize, serde::Deserialize, Debug)]
pub enum Command {
    #[command(subcommand)]
    Daemon(daemon::Command),

    #[command(subcommand)]
    Window(window::Command),

    #[command(subcommand)]
    Layer(layer_shell::Command),
    // TODO: Extra subcommand given by the user?
}

pub mod daemon {
    #[derive(clap::Parser, serde::Serialize, serde::Deserialize, Debug)]
    pub enum Command {
        Start {
            #[clap(long)]
            config: Option<String>,
        },
        Stop,
    }
}

pub mod window {
    #[derive(clap::Parser, serde::Serialize, serde::Deserialize, Debug)]
    pub enum Command {
        Create(Arguments),
        Toggle(Arguments),
    }

    #[derive(clap::Parser, serde::Serialize, serde::Deserialize, Debug)]
    pub struct Arguments {
        pub id: crate::config::window::Id,
    }
}

pub mod layer_shell {
    #[derive(clap::Parser, serde::Serialize, serde::Deserialize, Debug)]
    pub enum Command {
        Toggle(Arguments),
    }

    #[derive(clap::Parser, serde::Serialize, serde::Deserialize, Debug)]
    pub struct Arguments {
        pub id: crate::config::layer_shell::Namespace,
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Response {
    Success(String),
    Error(String),
}
