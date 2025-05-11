use clap::{Parser, arg, command};

use crate::config;

#[derive(Parser, serde::Serialize)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,

    /// Unknown arguments or everything after -- gets passed through to GTK.
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub gtk_options: Vec<String>,
}

#[derive(Parser, serde::Serialize, serde::Deserialize)]
pub enum Command {
    #[command(subcommand)]
    Daemon(DaemonCommand),

    #[command(subcommand)]
    Window(WindowCommand),
    // TODO: Extra subcommand given by the user?
}

#[derive(Parser, serde::Serialize, serde::Deserialize, Debug)]
pub enum DaemonCommand {
    Start {
        #[clap(long)]
        config: Option<String>,
    },
    Stop,
}

#[derive(Parser, serde::Serialize, serde::Deserialize, Debug)]
pub enum WindowCommand {
    Create(config::window::Header),
    Toggle(config::window::Header),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Response {
    Success(String),
    Error(String),
}
