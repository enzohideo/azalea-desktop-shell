use clap::{Parser, arg, command};

use crate::model;

#[derive(Parser, serde::Serialize)]
#[command(version, about, long_about = None)]
pub struct Arguments<Init>
where
    Init: clap::Subcommand + std::fmt::Debug,
{
    #[command(subcommand)]
    pub command: Command<Init>,

    /// Unknown arguments or everything after -- gets passed through to GTK.
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub gtk_options: Vec<String>,
}

#[derive(Parser, serde::Serialize, serde::Deserialize)]
pub enum Command<Init>
where
    Init: clap::Subcommand + std::fmt::Debug,
{
    #[command(subcommand)]
    Daemon(DaemonCommand),

    #[command(subcommand)]
    Window(WindowCommand<Init>),
}

#[derive(Parser, serde::Serialize, serde::Deserialize, Debug)]
pub enum DaemonCommand {
    Start,
    Stop,
}

#[derive(Parser, serde::Serialize, serde::Deserialize, Debug)]
pub enum WindowCommand<Init>
where
    Init: clap::Subcommand + std::fmt::Debug,
{
    Create(model::window::InitData<Init>),
}
