pub mod taskbar;

#[derive(Debug, Clone)]
pub struct Init<Model>
where
    Model: ModelExt,
{
    pub config: Model::Config,
}

impl<Model> Init<Model>
where
    Model: ModelExt,
{
    fn new(config: Model::Config) -> Self {
        Self { config }
    }
}

pub trait ModelExt {
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

        impl $crate::window::ModelExt for Model {
            type Config = Config;
        }

        pub type Init = $crate::window::Init<Model>;
    };
}

#[macro_export]
macro_rules! register_widgets {
    ($($window:ident, $model:ty;)+) => {
        use relm4::component::Connector;
        use relm4::prelude::Component;

        #[allow(dead_code)]
        pub enum WidgetWrapper {
            $($window(Connector<$model>)),+
        }

        #[allow(dead_code)]
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub enum ConfigWrapper {
            $($window(<$model as crate::window::ModelExt>::Config)),+
        }

        #[allow(dead_code)]
        impl ConfigWrapper {
            pub fn build_widget(self) -> (WidgetWrapper, gtk::Widget) {
                match self {
                    $(ConfigWrapper::$window(config) => {
                        let builder = <$model>::builder();
                        let widget = builder.root.clone();
                        let wrapper = WidgetWrapper::$window(
                            builder.launch($crate::window::Init::new(config))
                        );
                        (wrapper, gtk::glib::object::Cast::upcast::<gtk::Widget>(widget))
                    },)+
                }
            }
        }
    };
}
