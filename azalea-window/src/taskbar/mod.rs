use gtk::prelude::BoxExt;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, component};
use widget::{WidgetWrapper, build_widget};

pub mod widget;

crate::init! {
    Model {
        widgets: Vec<WidgetWrapper>,
    }

    Config {
        start: Vec<widget::Kind>,
        center: Vec<widget::Kind>,
        end: Vec<widget::Kind>,
    }

    Services {
        time: azalea_service::services::time::Service,
    }
}

#[component(pub)]
impl SimpleComponent for Model {
    type Init = Init;
    type Input = ();
    type Output = ();

    view! {
        gtk::Window {
            #[name(center_box)]
            gtk::CenterBox {
                #[name(start_widget)]
                #[wrap(Some)]
                set_start_widget = &gtk::Box {

                },

                #[name(center_widget)]
                #[wrap(Some)]
                set_center_widget = &gtk::Box {

                },

                #[name(end_widget)]
                #[wrap(Some)]
                set_end_widget = &gtk::Box {

                },
            }
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = Model { widgets: vec![] };
        let widgets = view_output!();

        for widget_kind in init.config.start {
            let (wrapper, widget) = build_widget(&init.services, widget_kind);
            model.widgets.push(wrapper);
            widgets.start_widget.append(&widget);
        }

        for widget_kind in init.config.center {
            let (wrapper, widget) = build_widget(&init.services, widget_kind);
            model.widgets.push(wrapper);
            widgets.center_widget.append(&widget);
        }

        for widget_kind in init.config.end {
            let (wrapper, widget) = build_widget(&init.services, widget_kind);
            model.widgets.push(wrapper);
            widgets.end_widget.append(&widget);
        }

        ComponentParts { model, widgets }
    }
}
