use crate::icon;

use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, component};

crate::init! {
    Model {
        separator: String,
    }

    Config {
        separator: Option<String>,
    }
}

#[component(pub)]
impl SimpleComponent for Model {
    type Init = Init;
    type Input = ();
    type Output = ();

    view! {
        gtk::Image {
            set_css_classes: &[
                "azalea-primary-fg",
            ],
            set_icon_name: Some(&model.separator),
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {
            separator: init
                .config
                .separator
                .unwrap_or(String::from(icon::SLASH_FORWARD)),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
