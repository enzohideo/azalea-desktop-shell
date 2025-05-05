pub mod taskbar;

#[derive(Debug, Clone, Default)]
pub struct Init<Config>
where
    Config: std::fmt::Debug + Clone + Default,
{
    pub config: Config,
}
