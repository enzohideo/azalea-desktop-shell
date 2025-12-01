use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::prelude::*;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt, component,
    prelude::FactoryVecDeque,
};

use crate::{
    component::image,
    factory, icon,
    service::{self, search::AppInfo},
};

crate::init! {
    Model {
        taskbar_image: relm4::Controller<image::Model>,
        apps: FactoryVecDeque<factory::search::apps::Model>,
        _service_handle: LocalListenerHandle,
    }

    Config {
    }
}

#[derive(Debug)]
pub enum Input {
    Search(String),
    SelectFirst,
    SearchResults(service::search::Output),
}

#[derive(Debug)]
pub enum CommandOutput {
    SetApplications(Vec<AppInfo>),
}

#[component(pub)]
impl Component for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();
    type CommandOutput = CommandOutput;

    view! {
        gtk::MenuButton {
            #[wrap(Some)]
            set_child= &gtk::Box {
                set_spacing: 12,

                #[local_ref]
                taskbar_image_widget -> gtk::Widget {
                    inline_css: "max-height: 30px;",
                    set_vexpand: false,
                },

                gtk::Label {
                    set_label: "Azalea"
                },
            },

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                set_has_arrow: false,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_width_request: 300,

                    gtk::Box {
                        set_spacing: 12,
                        set_css_classes: &[
                            "azalea-padding"
                        ],

                        gtk::Image {
                            set_icon_name: Some(icon::SEARCH),
                        },

                        gtk::Separator {
                            set_orientation: gtk::Orientation::Vertical,
                        },

                        gtk::Entry {
                            connect_changed[sender] => move |entry| {
                                sender.input(Input::Search(entry.text().to_string()));
                            },
                        },
                    },

                    gtk::ScrolledWindow {
                        set_propagate_natural_width: true,
                        set_height_request: 300,

                        set_css_classes: &[
                            "azalea-padding"
                        ],

                        #[local_ref]
                        search_result -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 5,
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {
            taskbar_image: image::Model::builder()
                .launch(image::Init {
                    fallback: None,
                    width: None,
                    height: Some(30),
                })
                .detach(),
            apps: FactoryVecDeque::builder()
                .launch(gtk::Box::default())
                .detach(),
            _service_handle: service::search::Service::forward_local(
                sender.input_sender().clone(),
                Input::SearchResults,
            ),
        };

        drop(model.taskbar_image.sender().send(image::Input::LoadBytes(
            include_bytes!("../../../../../../assets/azalea-logo.png").to_vec(),
        )));

        let (tx, rx) = flume::bounded(1);
        service::search::Service::send(service::search::Input::GetAllApplications(tx));
        sender.oneshot_command(async move {
            let mut applications = rx.recv_async().await.unwrap();
            applications.sort_by(|a, b| a.name.cmp(&b.name));
            CommandOutput::SetApplications(applications)
        });

        let taskbar_image_widget: &gtk::Widget = model.taskbar_image.widget().upcast_ref();
        let search_result = model.apps.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Search(message) => {
                self.apps
                    .broadcast(factory::search::apps::Input::FilterShowAll(message));
            }
            Input::SearchResults(_output) => todo!(),
            Input::SelectFirst => todo!(),
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CommandOutput::SetApplications(app_infos) => {
                let mut guard = self.apps.guard();

                for app_info in app_infos {
                    guard.push_back(app_info);
                }
            }
        }
    }
}
