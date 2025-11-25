use std::collections::HashMap;

use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::prelude::*;
use relm4::{
    Component, ComponentParts, ComponentSender, component, factory::FactoryHashMap, prelude::*,
};

use crate::{
    icon,
    service::{self, dbus::bluez::Device},
};

mod menu;

crate::init! {
    Model {
        devices_menu: FactoryHashMap<String, menu::BluetoothDeviceMenu>,
        _event_listener_handle: LocalListenerHandle,
    }

    Config {}
}

#[derive(Debug)]
pub enum Input {
    Connect(String, bool),
    Bluez(service::dbus::bluez::Output),
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
                set_valign: gtk::Align::Center,

                set_direction: gtk::ArrowType::Up,
                set_icon_name: icon::BLUETOOTH, // TODO: Change according to status

                #[wrap(Some)]
                set_popover = &gtk::Popover {
                    set_position: gtk::PositionType::Right,

                    gtk::Box {
                        #[local_ref]
                        devices_widget -> gtk::Box {
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
            devices_menu: FactoryHashMap::builder()
                .launch(gtk::Box::default())
                .forward(sender.input_sender(), |output| match output {
                    menu::Output::Connect(device, connect) => {
                        Input::Connect(device.address, connect)
                    }
                }),
            _event_listener_handle: service::dbus::bluez::Service::forward_local(
                sender.input_sender().clone(),
                Input::Bluez,
            ),
        };

        let (tx, rx) = flume::bounded(1);
        service::dbus::bluez::Service::send(service::dbus::bluez::Input::Devices(tx));
        service::dbus::bluez::Service::start();
        sender.oneshot_command(async move {
            let devices = rx.recv_async().await.unwrap();
            CommandOutput::SetDevices(devices)
        });

        let devices_widget = model.devices_menu.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Connect(address, connect) => {
                service::dbus::bluez::Service::send(service::dbus::bluez::Input::Connect(
                    address, connect,
                ));
            }
            Input::Bluez(output) => match output {
                service::dbus::bluez::Output::Connected(device_address, connected) => {
                    if let Some(mut menu_entry) = self.devices_menu.get_mut(&device_address) {
                        menu_entry.device.is_connected = connected;
                    }
                }
            },
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
                self.devices_menu.clear();
                for (address, device) in devices.into_iter() {
                    self.devices_menu.insert(address, device);
                }
            }
        }
    }
}
