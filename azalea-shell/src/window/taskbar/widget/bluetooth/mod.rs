use std::collections::HashMap;

use azalea_service::StaticHandler;
use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, component, prelude::*};

use crate::{
    icon,
    service::{self, dbus::bluez::Device},
};

mod menu;

crate::init! {
    Model {
        devices: HashMap<String, Device>,
        devices_menu: FactoryVecDeque<menu::BluetoothDeviceMenu>,
    }

    Config {}
}

#[derive(Debug)]
pub enum Input {
    Connect(Device, bool),
}

#[derive(Debug)]
pub enum CommandOutput {
    SetDevices(HashMap<String, Device>),
}

#[component(pub)]
impl Component for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();
    type CommandOutput = CommandOutput;

    view! {
        gtk::Box {
            gtk::MenuButton {
                set_hexpand: false,
                set_vexpand: false,

                set_direction: gtk::ArrowType::Up,
                set_icon_name: icon::BLUETOOTH, // TODO: Change according to status

                #[wrap(Some)]
                set_popover = &gtk::Popover {
                    set_position: gtk::PositionType::Right,

                    gtk::Box {
                        #[local_ref]
                        devices_menu_widget -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 5,
                        }
                    },
                },
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
            devices_menu: FactoryVecDeque::builder()
                .launch(gtk::Box::default())
                .forward(sender.input_sender(), |output| match output {
                    menu::Output::Connect(device, connect) => Input::Connect(device, connect),
                }),
        };

        let (tx, rx) = flume::bounded(1);
        service::dbus::bluez::Service::send(service::dbus::bluez::Input::Devices(tx));
        service::dbus::bluez::Service::start();
        sender.oneshot_command(async move {
            let devices = rx.recv_async().await.unwrap();
            CommandOutput::SetDevices(devices)
        });

        let devices_menu_widget = model.devices_menu.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Connect(device, connect) => {
                service::dbus::bluez::Service::send(service::dbus::bluez::Input::Connect(
                    device.address,
                    connect,
                ));
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
            CommandOutput::SetDevices(devices) => {
                self.devices = devices;
                let mut menu = self.devices_menu.guard();
                for device in self.devices.values() {
                    menu.push_back(device.clone());
                }
            }
        }
    }
}
