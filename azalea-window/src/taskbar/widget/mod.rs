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
        #[derive(Debug, Clone, clap::ValueEnum, serde::Serialize, serde::Deserialize)]
        pub enum Kind {
            $($window),+
        }

        #[allow(dead_code)]
        pub fn build_widget(dto: Kind) -> (WidgetWrapper, gtk::Widget) {
            match dto {
                $(Kind::$window => {
                    let builder = <$model>::builder();
                    let widget = builder.root.clone();
                    let wrapper = WidgetWrapper::$window(builder.launch(()));
                    (wrapper, gtk::glib::object::Cast::upcast::<gtk::Widget>(widget))
                },)+
            }
        }
    };
}

register_widgets!(
    Time, time::Model;
);
