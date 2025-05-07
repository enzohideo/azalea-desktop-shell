pub mod taskbar;

// TODO: Add service manager
#[derive(Debug, Clone, Default)]
pub struct Init<Config>
where
    Config: std::fmt::Debug + Clone + Default,
{
    pub config: Config,
}

impl<Config> Init<Config>
where
    Config: std::fmt::Debug + Clone + Default,
{
    fn new(config: Config) -> Self {
        Self { config }
    }
}

pub trait InitExt {
    type Config;
}
