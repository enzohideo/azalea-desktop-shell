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
        gtk::Label {
            set_label: &model.separator,
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {
            separator: init.config.separator.unwrap_or(format!("")),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
