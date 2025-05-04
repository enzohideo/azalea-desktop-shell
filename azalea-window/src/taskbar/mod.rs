use gtk::prelude::BoxExt;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, component};
use widget::{WidgetWrapper, build_widget};

pub mod widget;

#[derive(Debug, Clone, clap::Parser, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[clap(long)]
    pub start: Vec<widget::Kind>,

    #[clap(long)]
    pub center: Vec<widget::Kind>,

    #[clap(long)]
    pub end: Vec<widget::Kind>,
}

pub struct Model {
    widgets: Vec<WidgetWrapper>,
}

#[component(pub)]
impl SimpleComponent for Model {
    type Init = crate::Init<Config>;
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
            let (wrapper, widget) = build_widget(widget_kind);
            model.widgets.push(wrapper);
            widgets.start_widget.append(&widget);
        }

        for widget_kind in init.config.center {
            let (wrapper, widget) = build_widget(widget_kind);
            model.widgets.push(wrapper);
            widgets.center_widget.append(&widget);
        }

        for widget_kind in init.config.end {
            let (wrapper, widget) = build_widget(widget_kind);
            model.widgets.push(wrapper);
            widgets.end_widget.append(&widget);
        }

        ComponentParts { model, widgets }
    }
}
