use gtk::prelude::*;
use relm4::{
    FactorySender,
    prelude::{DynamicIndex, FactoryComponent},
};
use zbus::zvariant::OwnedObjectPath;

use crate::service::dbus::network_manager::proxy::{
    NetworkManagerDeviceProxy, NetworkManagerDeviceProxyBlocking, NetworkManagerProxyBlocking,
};

#[derive(Debug)]
pub struct Model {
    connection: Option<zbus::blocking::Connection>,
    name: String,
    proxy: Option<NetworkManagerDeviceProxyBlocking<'static>>,
}

#[derive(Debug)]
pub enum Input {
    Connect,
}

#[derive(Debug)]
pub enum Output {}

#[relm4::factory(pub)]
impl FactoryComponent for Model {
    type Init = OwnedObjectPath;
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

            // gtk::Button {
            //     set_halign: gtk::Align::End,
            //
            //     #[watch]
            //     set_icon_name: if self.device.is_connected {
            //         icon::PLUG_CONNECTED
            //     } else {
            //         icon::PLUG_DISCONNECTED
            //     },
            //
            //     #[watch]
            //     set_css_classes: if self.device.is_connected {
            //         &[ "azalea-primary-container" ]
            //     } else {
            //         &[]
            //     },
            //
            //     connect_clicked => Input::Connect
            // }
        }
    }

    fn init_model(device: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let connection = zbus::blocking::Connection::system().ok();

        let proxy = connection
            .as_ref()
            .and_then(|conn| NetworkManagerDeviceProxyBlocking::new(&conn, device).ok());

        Self {
            connection,
            name: proxy
                .as_ref()
                .and_then(|p| p.interface().ok())
                .unwrap_or(format!("unknown")),
            proxy,
        }
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            Input::Connect => {
                // TODO: Activate conneciton
                let _active_connection = self
                    .proxy
                    .as_ref()
                    .and_then(|p| p.active_connection().ok())
                    .and_then(|ac| {
                        NetworkManagerProxyBlocking::builder(&self.connection.as_ref().unwrap())
                            .path(ac)
                            .ok()
                    })
                    .and_then(|p| p.build().ok());
            }
        }
    }
}
