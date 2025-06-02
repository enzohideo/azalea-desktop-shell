pub mod component;
pub mod icon;
pub mod services;
pub mod taskbar;

#[derive(Debug, Clone)]
pub struct Init<Model>
where
    Model: InitExt,
{
    pub config: Model::Config,
}

impl<Model> Init<Model>
where
    Model: InitExt,
{
    fn new(config: Model::Config) -> Self {
        Self { config }
    }
}

pub trait InitExt {
    type Config;
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

        impl $crate::InitExt for Model {
            type Config = Config;
        }

        pub type Init = $crate::Init<Model>;
    };
}
