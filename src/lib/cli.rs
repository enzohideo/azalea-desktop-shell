use clap::{Parser, arg, command};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,

    /// Unknown arguments or everything after -- gets passed through to GTK.
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub gtk_options: Vec<String>,
}

#[derive(Parser)]
pub enum Command {
    #[command(subcommand)]
    Daemon(DaemonCommand),
}

#[derive(Parser)]
pub enum DaemonCommand {
    Start,
    Stop,
}
