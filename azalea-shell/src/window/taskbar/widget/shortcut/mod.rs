use gtk::{gio, prelude::*};
use relm4::{Component, ComponentParts, ComponentSender, component};

use crate::{icon, service::search::AppInfo};

crate::init! {
    Model {
        icon: Option<gio::Icon>,
        name: Option<String>,
        app_info: Option<gio::AppInfo>,
    }

    Config {
        desktop_entry: String,
    }
}

#[derive(Debug)]
pub enum Input {
    Click,
}

#[derive(Debug)]
pub enum CommandOutput {
    FindApplication(Vec<AppInfo>),
}

#[component(pub)]
impl Component for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();
    type CommandOutput = CommandOutput;

    view! {
        gtk::Button {
            gtk::Box {
                set_spacing: 8,


                gtk::Image {
                    set_from_gicon: model.icon
                        .as_ref()
                        .unwrap_or(&gio::ThemedIcon::from_names(&[icon::APPS]).upcast::<gio::Icon>())
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
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let app = gio::AppInfo::all().into_iter().find(|app| {
            app.id().map(|id| id.to_string()).unwrap_or_default() == init.config.desktop_entry
        });

        let model = Model {
            icon: app.as_ref().and_then(|app| app.icon()),
            name: app.as_ref().map(|app| app.name().to_string()),
            app_info: app,
        };

        if model.app_info.is_none() {
            azalea_log::warning!(Self, "Failed to find desktop entry");
        }

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Click => {
                if let Some(app_info) = &self.app_info {
                    let executable = app_info.executable();
                    match std::process::Command::new(&executable)
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn()
                    {
                        Ok(_) => azalea_log::debug!("Launched application: {:?}", executable),
                        Err(e) => azalea_log::warning!(
                            Self,
                            "Failed to launch application {:?}: {e}",
                            executable
                        ),
                    }
                }
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CommandOutput::FindApplication(app_infos) => {
                for app_info in app_infos {
                    println!("{app_info:#?}");
                }
            }
        }
    }
}
