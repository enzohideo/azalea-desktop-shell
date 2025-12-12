use azalea_service::StaticHandler;
use gtk::prelude::*;
use relm4::{
    FactorySender,
    prelude::{DynamicIndex, FactoryComponent},
};
use zbus::zvariant::OwnedObjectPath;

use crate::{
    icon,
    service::{self, dbus::network_manager::proxy::NetworkManagerSettingsConnectionProxyBlocking},
};

#[derive(Debug)]
pub struct Model {
    settings: OwnedObjectPath,
    name: String,
    #[allow(unused)]
    proxy: Option<NetworkManagerSettingsConnectionProxyBlocking<'static>>,
    active_connection: Option<OwnedObjectPath>,
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
pub enum Output {}

#[relm4::factory(pub)]
impl FactoryComponent for Model {
    /// Settings, Option<Active Connection>
    type Init = (OwnedObjectPath, Option<OwnedObjectPath>);
    type Input = Input;
    type Output = Output;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_spacing: 12,

            gtk::Label {
                set_halign: gtk::Align::Start,
                set_hexpand: true,
                set_label: &self.name,
            },

            gtk::Button {
                set_halign: gtk::Align::End,

                #[watch]
                set_icon_name: if self.active_connection.is_some() {
                    icon::PLUG_CONNECTED
                } else {
                    icon::PLUG_DISCONNECTED
                },

                #[watch]
                set_css_classes: if self.active_connection.is_some() {
                    &[ "azalea-primary-container" ]
                } else {
                    &[]
                },

                connect_clicked => Input::Toggle
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let connection = zbus::blocking::Connection::system().ok();

        let proxy = connection.as_ref().and_then(|conn| {
            NetworkManagerSettingsConnectionProxyBlocking::new(&conn, init.0.clone()).ok()
        });

        Self {
            connection,
            settings: init.0,
            name: proxy
                .as_ref()
                .and_then(|p| p.get_settings().ok())
                .and_then(|s| {
                    s.get("connection")
                        .and_then(|c| c.get("id"))
                        .map(|v| v.to_string().replace('"', ""))
                })
                .unwrap_or(format!("unknown")),
            active_connection: init.1,
            proxy,
        }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Input::Toggle => {
                if self.active_connection.is_some() {
                    self.update(Input::Disconnect, sender);
                } else {
                    self.update(Input::Connect, sender);
                }
            }
            Input::Connect => {
                service::dbus::network_manager::Service::send(
                    service::dbus::network_manager::Input::ActivateConnection {
                        connection: Some(self.settings.clone()),
                        device: None,
                        specific_object: None,
                    },
                );
            }
            Input::Disconnect => {
                if let Some(active_connection) = &self.active_connection {
                    service::dbus::network_manager::Service::send(
                        service::dbus::network_manager::Input::DeactivateConnection {
                            active_connection: active_connection.clone(),
                        },
                    );
                }
            }
        }
    }
}
