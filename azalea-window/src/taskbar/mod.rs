use relm4::{ComponentParts, ComponentSender, SimpleComponent, component};

#[derive(Debug, Clone, clap::Parser, serde::Serialize, serde::Deserialize)]
pub struct Init {}

pub struct Model {}

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
                    gtk::Label::new(Some("hey")) {}
                },

                #[name(end_widget)]
                #[wrap(Some)]
                set_end_widget = &gtk::Box {

                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {};
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
