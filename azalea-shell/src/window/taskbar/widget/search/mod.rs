use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::prelude::*;
use relm4::{
    Component, ComponentParts, ComponentSender, RelmWidgetExt, component, prelude::FactoryVecDeque,
};

use crate::{
    factory, icon,
    service::{self, search::AppInfo},
};

crate::init! {
    Model {
        search: String,
        apps: FactoryVecDeque<factory::search::apps::Model>,
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

            #[wrap(Some)]
            set_child= &gtk::Box {
                gtk::Image {
                    set_icon_name: Some(icon::SEARCH),
                },

                gtk::Label {
                    #[watch]
                    set_label: &model.search,
                },
            },

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                set_has_arrow: false,

                connect_closed => move |_| {
                    entry_clone.set_text("");
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[local_ref]
                    entry -> gtk::Entry {
                        set_vexpand: false,
                        set_width_request: 0,
                        set_width_chars: 0,
                        inline_css: "color: transparent; max-width: 0;",
                        connect_activate => Input::SelectFirst,
                        connect_changed[sender] => move |entry| {
                            sender.input(Input::Search(entry.text().to_string()));
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
            search: format!(""),
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

        let entry = gtk::Entry::new();
        let entry_clone = entry.clone();
        let search_result = model.apps.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Search(message) => {
                self.search = message.clone();
                self.apps
                    .broadcast(factory::search::apps::Input::Filter(message));
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
