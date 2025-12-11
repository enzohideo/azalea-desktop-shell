use crate::service;
use azalea_service::LocalStaticHandler;
use gtk::prelude::ButtonExt;
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
            gtk::Button {
                set_label: "Suspend",
                connect_clicked => Self::Input::Suspend
            },
            gtk::Button {
                set_label: "Hibernate",
                connect_clicked => Self::Input::Hibernate
            },
            gtk::Button {
                set_label: "Reboot",
                connect_clicked => Self::Input::Reboot
            },
            gtk::Button {
                set_label: "Shutdown",
                connect_clicked => Self::Input::PowerOff
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
