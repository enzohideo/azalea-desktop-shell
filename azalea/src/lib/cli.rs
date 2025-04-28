use bincode::{Decode, Encode};
use clap::{Parser, arg, command};

use crate::model;

#[derive(Parser, Encode, Decode)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,

    /// Unknown arguments or everything after -- gets passed through to GTK.
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub gtk_options: Vec<String>,
}

#[derive(Parser, Encode, Decode)]
pub enum Command {
    #[command(subcommand)]
    Daemon(DaemonCommand),

    #[command(subcommand)]
    Window(WindowCommand),
}

#[derive(Parser, Encode, Decode, Debug)]
pub enum DaemonCommand {
    Start,
    Stop,
}

#[derive(Parser, Encode, Decode, Debug)]
pub enum WindowCommand {
    Create(model::Window),
}
