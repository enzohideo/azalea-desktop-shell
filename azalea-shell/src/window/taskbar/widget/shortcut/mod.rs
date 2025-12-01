use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, component};

crate::init! {
    Model {
        icon: Option<String>,
        name: Option<String>,
        executable: String,
    }

    Config {
        icon: Option<String>,
        name: Option<String>,
        executable: String,
    }
}

#[derive(Debug)]
pub enum Input {
    Click,
}

#[component(pub)]
impl SimpleComponent for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();

    view! {
        gtk::Button {
            gtk::Box {
                set_spacing: 8,

                gtk::Image {
                    set_icon_name: model.icon.as_deref(),
                },
                gtk::Label {
                    set_label: &model.name.as_deref().unwrap_or(""),
                }
            },

            connect_clicked => Input::Click,
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {
            name: init.config.name,
            icon: init.config.icon,
            executable: init.config.executable,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::Click => match std::process::Command::new(&self.executable).spawn() {
                Ok(_) => azalea_log::debug!("Launched application: {:?}", self.executable),
                Err(e) => {
                    azalea_log::warning!("Failed to launch application {:?}: {e}", self.executable)
                }
            },
        }
    }
}
