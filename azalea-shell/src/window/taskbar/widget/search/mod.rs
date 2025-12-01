use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::{gdk, glib, prelude::*};
use gtk4_layer_shell::LayerShell;
use relm4::{Component, ComponentParts, ComponentSender, component, prelude::FactoryVecDeque};

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
        gtk::Button {
            set_valign: gtk::Align::Center,
            #[watch]
            set_css_classes: if model.search.len() > 0 {
                &[
                    "azalea-primary-container",
                    "azalea-circle-bubble",
                    "azalea-primary-border",
                    "azalea-secondary-container-hover",
                ]
            } else { &[] },

            #[wrap(Some)]
            set_child= &gtk::Box {
                set_spacing: 8,
                set_valign: gtk::Align::Center,

                gtk::Image {
                    set_icon_name: Some(icon::SEARCH),
                },

                gtk::Label {
                    #[watch]
                    set_label: &model.search,
                },
            },

            connect_clicked => move |_| {
                window.set_visible(!window.get_visible());
            },
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

        relm4::view! {
            window = gtk::Window {
                init_layer_shell: (),

                set_layer: gtk4_layer_shell::Layer::Overlay,

                set_anchor: (gtk4_layer_shell::Edge::Top, true),
                set_anchor: (gtk4_layer_shell::Edge::Bottom, true),
                set_anchor: (gtk4_layer_shell::Edge::Left, true),
                set_anchor: (gtk4_layer_shell::Edge::Right, true),

                set_keyboard_mode: gtk4_layer_shell::KeyboardMode::OnDemand,

                set_visible: false,

                connect_visible_notify => move |this| {
                    if !this.get_visible() {
                        entry_clone.set_text("");
                    }
                },

                add_controller = gtk::EventControllerKey {
                    connect_key_pressed => move |this, key, _code, _modifier| {
                        match key {
                            gdk::Key::Escape => {
                                if let Some(widget) = this.widget(){
                                    widget.set_visible(false);
                                }
                                glib::Propagation::Stop
                            },
                            _ => glib::Propagation::Proceed,
                        }
                    },
                },

                add_css_class: "azalea-transparent",

                gtk::Box {
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 24,

                    set_width_request: 300,

                    gtk::Box {
                        set_spacing: 12,
                        set_css_classes: &[
                            "azalea-surface",
                            "azalea-semi-transparent",
                            "azalea-bubble",
                            "azalea-primary-border",
                            "azalea-padding"
                        ],

                        gtk::Image {
                            set_icon_name: Some(icon::SEARCH),
                        },

                        gtk::Separator {
                            set_orientation: gtk::Orientation::Vertical,
                        },

                        #[local_ref]
                        entry -> gtk::Entry {
                            connect_activate => Input::SelectFirst,
                            connect_changed[sender] => move |entry| {
                                sender.input(Input::Search(entry.text().to_string()));
                            },
                        },
                    },

                    gtk::ScrolledWindow {
                        set_propagate_natural_width: true,
                        set_propagate_natural_height: true,

                        set_css_classes: &[
                            "azalea-surface",
                            "azalea-semi-transparent",
                            "azalea-bubble",
                            "azalea-primary-border",
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
        };

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
