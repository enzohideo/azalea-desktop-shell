pub mod bluetooth;
pub mod media;
pub mod network;
pub mod search;
pub mod time;

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

register_widgets!(
    Bluetooth, bluetooth::Model;
    Media, media::Model;
    Network, network::Model;
    Search, search::Model;
    Time, time::Model;
);
