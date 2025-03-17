use clap::{Parser, arg, command};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    /// Unknown arguments or everything after -- gets passed through to GTK.
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub gtk_options: Vec<String>,
}
