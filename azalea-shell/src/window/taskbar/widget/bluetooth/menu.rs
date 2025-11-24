use gtk::prelude::{BoxExt, ButtonExt, WidgetExt};
use relm4::{
    FactorySender,
    prelude::{DynamicIndex, FactoryComponent},
};

use crate::{icon, service::dbus::bluez::Device};

#[derive(Debug)]
pub struct BluetoothDeviceMenu {
    device: Device,
}

#[derive(Debug)]
pub enum Input {
    Connect,
    Connected(bool),
}

#[derive(Debug)]
pub enum Output {
    Connect(Device, bool),
}

#[relm4::factory(pub)]
impl FactoryComponent for BluetoothDeviceMenu {
    type Init = Device;
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
                set_label: self.device.name.as_ref().unwrap_or(&String::from("unknown")),
            },

            gtk::Button {
                set_halign: gtk::Align::End,
                set_icon_name: if self.device.is_connected {
                    icon::PLUG_CONNECTED
                } else {
                    icon::PLUG_DISCONNECTED
                },
                connect_clicked => Input::Connect
            }
        }
    }

    fn init_model(device: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { device }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Input::Connect => drop(sender.output(Output::Connect(
                self.device.clone(),
                !self.device.is_connected,
            ))),
            Input::Connected(is_connected) => self.device.is_connected = is_connected,
        }
    }
}
