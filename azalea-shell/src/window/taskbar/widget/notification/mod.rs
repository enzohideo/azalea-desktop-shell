use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, component, factory::FactoryHashMap};

use crate::{
    factory,
    service::{self, dbus::notification},
};

crate::init! {
    Model {
        notifications: FactoryHashMap<u32, factory::notification::Model>,
        _service_handle: LocalListenerHandle,
    }

    Config {
    }
}

#[derive(Debug)]
pub enum Input {
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
            set_child= &gtk::Label {
                set_label: "notifications",
            },

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                set_position: gtk::PositionType::Right,

                #[local_ref]
                notifications_widget -> gtk::Box {
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
            Input::Close(id) => {
                self.notifications.remove(&id);
                // TODO: Call Close method in notification service
            }
            Input::Notifications(message) => match message {
                notification::Output::Notification(notification) => {
                    self.notifications.insert(notification.id, notification);
                }
            },
        }
    }
}
