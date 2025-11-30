use gtk::prelude::*;
use relm4::{FactorySender, prelude::*};

use crate::service::dbus::notification;

#[derive(Debug)]
pub struct Model {
    notification: notification::service::Notification,
}

#[derive(Debug)]
pub enum Input {
    Close,
    // TODO: Action(String)
}

#[derive(Debug)]
pub enum Output {
    Close(u32),
    // TODO: Action(String)
}

#[relm4::factory(pub)]
impl FactoryComponent for Model {
    type Index = u32;
    type Init = notification::service::Notification;
    type Input = Input;
    type Output = Output;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::Label {
                #[watch]
                set_label: &self.notification.app_name,
            },

            gtk::Label {
                #[watch]
                set_label: &self.notification.body,
            },

            gtk::Button {
                set_label: "X",
                connect_clicked => Input::Close
            },
        }
    }

    fn init_model(notification: Self::Init, _index: &u32, _sender: FactorySender<Self>) -> Self {
        Self { notification }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Input::Close => drop(sender.output(Output::Close(self.notification.id))),
        }
    }
}
