use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, component, factory::FactoryHashMap};

use crate::{
    factory, icon,
    service::{self, dbus::notification},
};

crate::init! {
    Model {
        num_unread: usize,
        latest_notification: Option<notification::service::Notification>,
        notifications: FactoryHashMap<u32, factory::notification::Model>,
        _service_handle: LocalListenerHandle,
    }

    Config {
    }
}

#[derive(Debug)]
pub enum Input {
    ClearLatest,
    Close(u32),
    Notifications(notification::Output),
}

#[component(pub)]
impl SimpleComponent for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();

    view! {
        gtk::MenuButton {
            #[wrap(Some)]
            set_child= &gtk::Box {
                set_spacing: 8,
                set_hexpand: true,

                gtk::Revealer {
                    set_transition_type: gtk::RevealerTransitionType::SlideLeft,
                    set_transition_duration: 1000,

                    #[watch]
                    set_reveal_child: model.latest_notification.is_some(),

                    gtk::Label {
                        set_max_width_chars: 30,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,

                        #[watch]
                        set_label: if model.latest_notification.is_some() {
                            &model.latest_notification.as_ref().unwrap().summary
                        } else { "" }
                    },
                },

                gtk::Image {
                    set_icon_name: Some(icon::BELL),
                },

                gtk::Label {
                    #[watch]
                    set_visible: model.num_unread > 0,

                    #[watch]
                    set_label: &format!("{}", model.num_unread),
                },
            },

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                set_position: gtk::PositionType::Right,

                connect_notify: (Some("visible"), move |this, _| {
                    if !this.get_visible() {
                        drop(sender.input_sender().send(Input::ClearLatest));
                    }
                }),

                #[local_ref]
                notifications_widget -> gtk::Box {
                    add_css_class: "azalea-transparent",
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 5,
                }
            },
        },
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {
            latest_notification: None,
            num_unread: 0,
            _service_handle: service::dbus::notification::Service::forward_local(
                sender.input_sender().clone(),
                Input::Notifications,
            ),
            notifications: FactoryHashMap::builder()
                .launch(gtk::Box::default())
                .forward(sender.input_sender(), |output| match output {
                    factory::notification::Output::Close(id) => Input::Close(id),
                }),
        };

        let notifications_widget = model.notifications.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::ClearLatest => {
                self.latest_notification = None;
            }
            Input::Close(id) => {
                self.notifications.remove(&id);
                self.num_unread -= 1;
                // TODO: Call Close method in notification service
            }
            Input::Notifications(message) => match message {
                notification::Output::Notification(notification) => {
                    self.num_unread += 1;
                    self.latest_notification = Some(notification.clone());
                    self.notifications.insert(notification.id, notification);
                }
            },
        }
    }
}
