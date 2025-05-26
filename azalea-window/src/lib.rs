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

#[macro_export]
macro_rules! init {
    (
        Model {
            $($model_vis: vis $model_name: ident: $model_type: ty,)*
        }

        Config {
            $($config_name: ident: $config_type: ty,)*
        }

        Services {
            $($service_name: ident: $service_model:ty,)*
        }
    ) => {
        pub struct Model {
            $($model_vis $model_name: $model_type,)*
        }

        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
        pub struct Config {
            $(
                pub $config_name: $config_type,
            )*
        }

        azalea_service::services! {
            $(optional $service_name: $service_model;)*
        }

        impl $crate::InitExt for Model {
            type Config = Config;
            type Services = Services;
        }

        pub type Init = $crate::Init<Model>;
    };
}
