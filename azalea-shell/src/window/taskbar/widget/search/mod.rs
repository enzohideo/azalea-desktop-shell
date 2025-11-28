use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::prelude::*;
use relm4::{
    Component, ComponentParts, ComponentSender, component, prelude::FactoryVecDeque,
};

use crate::{
    icon,
    service::{self, search::AppInfo},
};

mod apps;

crate::init! {
    Model {
        apps: FactoryVecDeque<apps::Entry>,
        _service_handle: LocalListenerHandle,
    }

    Config {
        // TODO: Use this to determine which order display results
        top_down: bool,
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
            set_icon_name: icon::SEARCH,

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                set_has_arrow: false,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Box {
                        set_spacing: 12,

                        gtk::Image {
                            set_icon_name: Some(icon::SEARCH),
                        },

                        gtk::Entry {
                            connect_activate => Input::SelectFirst,
                            connect_changed[sender] => move |entry| {
                                sender.input(Input::Search(entry.text().to_string()));
                            },
                        },
                    },

                    gtk::ScrolledWindow {
                        set_propagate_natural_width: true,
                        set_propagate_natural_height: true,

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
            apps: FactoryVecDeque::builder()
                .launch(gtk::Box::default())
                .detach(),
            _service_handle: service::search::Service::forward_local(
                sender.input_sender().clone(),
                Input::SearchResults,
            ),
        };

        let (tx, rx) = flume::bounded(1);
        service::search::Service::send(service::search::Input::GetAllApplications(tx));
        sender.oneshot_command(async move {
            let mut applications = rx.recv_async().await.unwrap();
            applications.sort_by(|a, b| a.name.cmp(&b.name));
            CommandOutput::SetApplications(applications)
        });

        let search_result = model.apps.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Search(message) => {
                self.apps.broadcast(apps::Input::Filter(message));
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
