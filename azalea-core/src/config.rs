use crate::model;

pub struct Config<Init>
where
    Init: clap::Subcommand + bincode::enc::Encode + bincode::de::Decode<()> + std::fmt::Debug,
{
    pub windows: Vec<model::window::Init<Init>>,
}
