use gtk::prelude::BoxExt;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, component};

#[derive(Debug, Clone, clap::Parser, serde::Serialize, serde::Deserialize)]
pub struct Init {}

#[derive(Debug)]
pub enum Input {
    UpdateTime(chrono::DateTime<chrono::Local>),
}

pub struct Model {
    // TODO: Move time to separate widget
    time: chrono::DateTime<chrono::Local>,
}

#[component(pub)]
impl SimpleComponent for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();

    view! {
        gtk::Window {
            #[name(center_box)]
            gtk::CenterBox {
                #[name(start_widget)]
                #[wrap(Some)]
                set_start_widget = &gtk::Box {
                    gtk::Label::new(Some("TODO: start widgets")) {}
                },

                #[name(center_widget)]
                #[wrap(Some)]
                set_center_widget = &gtk::Box {
                    gtk::Label::new(Some("TODO: center widgets")) {}
                },

                #[name(end_widget)]
                #[wrap(Some)]
                set_end_widget = &gtk::Box {
                    set_spacing: 12,

                    gtk::Label {
                        #[watch]
                        set_label: &format!("{}", model.time.format("%d/%m/%y"))
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &format!("{}", model.time.format("%H:%M:%S"))
                    },
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {
            time: chrono::Local::now(),
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::UpdateTime(date_time) => self.time = date_time,
        }
    }
}
