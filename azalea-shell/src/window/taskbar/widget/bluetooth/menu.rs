use gtk::prelude::{BoxExt, ButtonExt, WidgetExt};
use relm4::{FactorySender, prelude::FactoryComponent};

use crate::{icon, service::dbus::bluez::Device};

#[derive(Debug)]
pub struct BluetoothDeviceMenu {
    pub device: Device,
}

#[derive(Debug)]
pub enum Input {
    Connect,
}

#[derive(Debug)]
pub enum Output {
    Connect(Device, bool),
}

#[relm4::factory(pub)]
impl FactoryComponent for BluetoothDeviceMenu {
    type Index = String;
    type Init = Device;
    type Input = Input;
    type Output = Output;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_spacing: 12,

            gtk::Image {
                set_icon_name: Some(self.device.icon.as_deref().unwrap_or(icon::BLUETOOTH)),
            },

            gtk::Label {
                set_halign: gtk::Align::Start,
                set_hexpand: true,
                set_label: self.device.name.as_ref().unwrap_or(&String::from("unknown")),
            },

            gtk::Button {
                set_halign: gtk::Align::End,

                #[watch]
                set_icon_name: if self.device.is_connected {
                    icon::PLUG_CONNECTED
                } else {
                    icon::PLUG_DISCONNECTED
                },

                #[watch]
                set_css_classes: if self.device.is_connected {
                    &[ "primary-container-button" ]
                } else {
                    &[]
                },

                connect_clicked => Input::Connect
            }
        }
    }

    fn init_model(device: Self::Init, _index: &String, _sender: FactorySender<Self>) -> Self {
        Self { device }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Input::Connect => drop(sender.output(Output::Connect(
                self.device.clone(),
                !self.device.is_connected,
            ))),
        }
    }
}
