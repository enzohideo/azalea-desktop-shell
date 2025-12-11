use crate::{icon, service};
use azalea_service::LocalStaticHandler;
use gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, component};
use std::fmt::Debug;

pub struct Model {}

#[component(pub)]
impl SimpleComponent for Model {
    type Input = service::dbus::login::Input;
    type Output = ();
    type Init = ();

    view! {
        gtk::Box {
            set_hexpand: true,
            set_halign: gtk::Align::Fill,

            gtk::Button {
                set_hexpand: true,
                set_halign: gtk::Align::Fill,

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_halign: gtk::Align::Center,
                    set_spacing: 6,

                    gtk::Image {
                        set_icon_name: Some(icon::SUSPEND),
                    },

                    gtk::Label {
                        set_label: "Suspend",
                    },
                },

                connect_clicked => Self::Input::Suspend
            },
            gtk::Button {
                set_hexpand: true,
                set_halign: gtk::Align::Fill,

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_halign: gtk::Align::Center,
                    set_spacing: 6,

                    gtk::Image {
                        set_icon_name: Some(icon::REBOOT),
                    },

                    gtk::Label {
                        set_label: "Reboot",
                    },
                },

                connect_clicked => Self::Input::Reboot
            },
            gtk::Button {
                set_hexpand: true,
                set_halign: gtk::Align::Fill,

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_halign: gtk::Align::Center,
                    set_spacing: 6,

                    gtk::Image {
                        set_icon_name: Some(icon::SHUTDOWN),
                    },

                    gtk::Label {
                        set_label: "Shutdown",
                    },
                },

                connect_clicked => Self::Input::PowerOff
            },
            gtk::Button {
                set_hexpand: true,
                set_halign: gtk::Align::Fill,

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_halign: gtk::Align::Center,
                    set_spacing: 6,

                    gtk::Image {
                        set_icon_name: Some(icon::HIBERNATE),
                    },

                    gtk::Label {
                        set_label: "Hibernate",
                    },
                },

                connect_clicked => Self::Input::Hibernate
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};

        let widgets = view_output!();

        service::dbus::login::Service::start();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>) {
        service::dbus::login::Service::send(input);
    }
}
