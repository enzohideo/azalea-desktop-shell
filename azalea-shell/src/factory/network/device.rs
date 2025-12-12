use azalea_service::StaticHandler;
use gtk::prelude::*;
use relm4::{
    FactorySender,
    prelude::{DynamicIndex, FactoryComponent},
};
use zbus::zvariant::OwnedObjectPath;

use crate::{
    icon,
    service::{
        self,
        dbus::network_manager::proxy::{NMDeviceState, NetworkManagerDeviceProxyBlocking},
    },
};

#[derive(Debug)]
pub struct Model {
    device: OwnedObjectPath,
    name: String,
    proxy: Option<NetworkManagerDeviceProxyBlocking<'static>>,
    is_activated: bool,
    visible: bool,
    #[allow(unused)]
    connection: Option<zbus::blocking::Connection>,
}

#[derive(Debug)]
pub enum Input {
    Toggle,
    Connect,
    Disconnect,
}

#[derive(Debug)]
pub enum CommandOutput {
    State(service::dbus::network_manager::proxy::NMDeviceState),
}

#[derive(Debug)]
pub enum Output {}

#[relm4::factory(pub)]
impl FactoryComponent for Model {
    type Init = OwnedObjectPath;
    type Input = Input;
    type Output = Output;
    type CommandOutput = CommandOutput;
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_visible: self.visible,
            set_spacing: 12,

            gtk::Label {
                set_halign: gtk::Align::Start,
                set_hexpand: true,
                set_label: &self.name,
            },

            gtk::Button {
                set_halign: gtk::Align::End,

                #[watch]
                set_icon_name: if self.is_activated {
                    icon::PLUG_CONNECTED
                } else {
                    icon::PLUG_DISCONNECTED
                },

                #[watch]
                set_css_classes: if self.is_activated {
                    &[ "azalea-primary-container" ]
                } else {
                    &[]
                },

                connect_clicked => Input::Toggle
            }
        }
    }

    fn init_model(device: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let connection = zbus::blocking::Connection::system().ok();

        let proxy = connection
            .as_ref()
            .and_then(|conn| NetworkManagerDeviceProxyBlocking::new(&conn, device.clone()).ok());

        let state_stream = proxy.as_ref().map(|p| p.receive_state_changed());
        if let Some(mut state_stream) = state_stream {
            let cmd_sender = sender.command_sender().clone();
            relm4::spawn_blocking(move || {
                while let Some(prop) = state_stream.next() {
                    if let Ok(state) = prop.get() {
                        if let Err(_) = cmd_sender.send(CommandOutput::State(state)) {
                            break;
                        }
                    }
                }
            });
        }

        Self {
            connection,
            device,
            name: proxy
                .as_ref()
                .and_then(|p| p.interface().ok())
                .unwrap_or(format!("unknown")),
            is_activated: proxy
                .as_ref()
                .and_then(|p| p.state().ok())
                .map(|s| s == NMDeviceState::NMDeviceStateActivated)
                .unwrap_or(false),
            visible: proxy
                .as_ref()
                .and_then(|p| p.device_type().ok())
                .map(|t| match t {
                    _ => true,
                })
                .unwrap_or(false),
            proxy,
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Input::Toggle => {
                if self.is_activated {
                    self.update(Input::Disconnect, sender);
                } else {
                    self.update(Input::Connect, sender);
                }
            }
            Input::Connect => {
                service::dbus::network_manager::Service::send(
                    service::dbus::network_manager::Input::ActivateConnection {
                        connection: None,
                        device: Some(self.device.clone()),
                        specific_object: None,
                    },
                );
            }
            Input::Disconnect => {
                if let Some(proxy) = &self.proxy {
                    drop(proxy.disconnect());
                };
            }
        }
    }

    fn update_cmd(&mut self, message: Self::CommandOutput, _sender: FactorySender<Self>) {
        match message {
            CommandOutput::State(nmdevice_state) => match nmdevice_state {
                NMDeviceState::NMDeviceStateActivated => self.is_activated = true,
                _ => self.is_activated = false,
            },
        }
    }
}
