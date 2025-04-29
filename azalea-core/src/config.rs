use crate::model;

pub struct Config<Init>
where
    Init: clap::Subcommand + std::fmt::Debug,
{
    pub windows: Vec<model::window::Init<Init>>,
}
