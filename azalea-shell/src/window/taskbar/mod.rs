//! # Azalea taskbar window

use gtk::prelude::BoxExt;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, component};
use widget::WidgetWrapper;

pub mod widget;

crate::init! {
    Model {
        widgets: Vec<WidgetWrapper>,
    }

    Config {
        spacing: i32,
        start: Vec<widget::ConfigWrapper>,
        center: Vec<widget::ConfigWrapper>,
        end: Vec<widget::ConfigWrapper>,
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

        for widget_config in init.config.start {
            let (wrapper, widget) = widget_config.build_widget();
            model.widgets.push(wrapper);
            widgets.start_widget.append(&widget);
            widgets.start_widget.set_spacing(init.config.spacing);
        }

        for widget_config in init.config.center {
            let (wrapper, widget) = widget_config.build_widget();
            model.widgets.push(wrapper);
            widgets.center_widget.append(&widget);
            widgets.center_widget.set_spacing(init.config.spacing);
        }

        for widget_config in init.config.end {
            let (wrapper, widget) = widget_config.build_widget();
            model.widgets.push(wrapper);
            widgets.end_widget.append(&widget);
            widgets.end_widget.set_spacing(init.config.spacing);
        }

        ComponentParts { model, widgets }
    }
}
