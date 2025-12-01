use gtk::{gdk, glib, prelude::*};
use relm4::{FactorySender, prelude::*};

use crate::{component::image, service::dbus::notification};

pub struct Model {
    notification: notification::service::Notification,
    image: relm4::Controller<image::Model>,
    has_image: bool,
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
            gtk::Frame {
                set_visible: self.has_image,
                set_child: Some(self.image.widget()),
            },

            gtk::Box {
                set_css_classes: &[
                    "azalea-padding",
                ],
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 12,
                set_vexpand: true,
                set_hexpand: true,

                gtk::Label {
                    set_css_classes: &[
                        "azalea-primary-fg",
                    ],
                    #[watch]
                    set_label: &self.notification.summary,

                    set_max_width_chars: 30,
                    set_wrap: true,
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Start,
                },

                gtk::Label {
                    #[watch]
                    set_label: &self.notification.body,

                    set_max_width_chars: 30,
                    set_wrap: true,
                    set_halign: gtk::Align::Start,
                    set_valign: gtk::Align::Center,
                },
            },

            gtk::Button {
                set_css_classes: &[
                    "azalea-padding",
                ],
                set_halign: gtk::Align::End,
                set_label: "X",
                connect_clicked => Input::Close
            },
        }
    }

    fn init_model(notification: Self::Init, _index: &u32, _sender: FactorySender<Self>) -> Self {
        let mut model = Self {
            has_image: false,
            image: image::Model::builder()
                .launch(image::Init {
                    fallback: None,
                    width: None,
                    height: Some(100),
                })
                .detach(),
            notification: notification.clone(),
        };

        match notification.image.clone() {
            Some(image) => {
                model.has_image = true;
                match image {
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
                }
            }
            None => {
                if notification.app_icon != "" {
                    model.has_image = true;
                    println!("{}", notification.app_icon);
                    drop(
                        model
                            .image
                            .sender()
                            .send(image::Input::LoadImage(notification.app_icon)),
                    );
                }
            }
        };

        model
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Input::Close => drop(sender.output(Output::Close(self.notification.id))),
        }
    }
}
