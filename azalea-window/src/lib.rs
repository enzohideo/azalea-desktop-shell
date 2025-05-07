pub mod taskbar;

// TODO: Add service manager
#[derive(Debug, Clone)]
pub struct Init<Model>
where
    Model: InitExt,
{
    pub config: Model::Config,
    pub services: Model::Services,
}

impl<Model> Init<Model>
where
    Model: InitExt,
{
    fn new(services: Model::Services, config: Model::Config) -> Self {
        Self { services, config }
    }
}

pub trait InitExt {
    type Config;
    type Services;
}
