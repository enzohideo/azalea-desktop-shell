pub mod taskbar;

#[derive(Debug, Clone)]
pub struct Init<Config>
where
    Config: std::fmt::Debug + Clone,
{
    pub config: Config,
}
