use std::collections::HashMap;

use azalea_service::StaticHandler;
use relm4::{Component, ComponentParts, ComponentSender, component};

use crate::service::{self, dbus::bluez::Device};

crate::init! {
    Model {
        devices: HashMap<String, HashMap<String, Device>>,
    }

    Config {}
}

#[derive(Debug)]
pub enum Input {}

#[derive(Debug)]
pub enum CommandOutput {
    SetDevices(HashMap<String, HashMap<String, Device>>),
}

#[component(pub)]
impl Component for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();
    type CommandOutput = CommandOutput;

    view! {
        gtk::Box {
            gtk::Label {
                #[watch]
                set_label: &model.devices.keys().fold(String::new(),|acc, key| acc + &key)
            },
        },
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {
            devices: Default::default(),
        };

        let (tx, rx) = flume::bounded(1);
        service::dbus::bluez::Service::send(service::dbus::bluez::Input::Devices(tx));
        service::dbus::bluez::Service::start();
        sender.oneshot_command(async move {
            let devices = rx.recv_async().await.unwrap();
            CommandOutput::SetDevices(devices)
        });

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CommandOutput::SetDevices(devices) => self.devices = devices,
        }
    }
}
