use gtk::{gdk, glib, prelude::*};
use relm4::{FactorySender, prelude::*};

use crate::{component::image, service::dbus::notification};

pub struct Model {
    notification: notification::service::Notification,
    image: relm4::Controller<image::Model>,
    // image: Option<gdk::Texture>,
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
            append: self.image.widget(),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Label {
                    #[watch]
                    set_label: &self.notification.summary,
                },

                gtk::Label {
                    #[watch]
                    set_label: &self.notification.body,
                },
            },

            gtk::Button {
                set_valign: gtk::Align::Center,
                set_label: "X",
                connect_clicked => Input::Close
            },
        }
    }

    fn init_model(notification: Self::Init, _index: &u32, _sender: FactorySender<Self>) -> Self {
        let model = Self {
            image: image::Model::builder()
                .launch(image::Init {
                    fallback: None,
                    width: None,
                    height: Some(30),
                })
                .detach(),
            notification: notification.clone(),
        };

        match notification.image.clone() {
            Some(image) => match image {
                notification::service::Image::Data {
                    width,
                    height,
                    rowstride,
                    has_alpha,
                    bits_per_sample,
                    channels: _,
                    data,
                } => {
                    let pixbuf = gdk::gdk_pixbuf::Pixbuf::from_bytes(
                        &glib::Bytes::from_owned(data),
                        gtk::gdk_pixbuf::Colorspace::Rgb,
                        has_alpha,
                        bits_per_sample,
                        width,
                        height,
                        rowstride,
                    );

                    drop(model.image.sender().send(image::Input::LoadPixbuf(pixbuf)));
                }
                notification::service::Image::Path(path) => {
                    drop(model.image.sender().send(image::Input::LoadImage(path)));
                }
            },
            None => (),
        };

        model
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Input::Close => drop(sender.output(Output::Close(self.notification.id))),
        }
    }
}
