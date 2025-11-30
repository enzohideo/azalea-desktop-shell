use azalea_service::{ListenerHandle, StaticHandler};
use gtk::prelude::*;
use relm4::RelmWidgetExt;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, component};

use crate::icon;

use crate::service;

crate::init! {
    Model {
        brightness: f64,
        _service_handle: ListenerHandle,
    }

    Config {
    }
}

#[derive(Debug)]
pub enum Input {
    Scroll(f64),
    Brightness(service::brightness::Output),
}

#[component(pub)]
impl SimpleComponent for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();

    view! {
        gtk::Box {
            set_spacing: 8,

            gtk::Image {
                set_icon_name: Some(icon::BRIGHTNESS),
            },

            gtk::Label {
                #[watch]
                set_label: &format!("{}%", model.brightness_percent()),
            },

            gtk::Frame {
                set_width_request: 100,
                set_height_request: 5,
                set_vexpand: false,
                set_valign: gtk::Align::Center,

                #[watch]
                inline_css: &format!(
                    "background-image: linear-gradient(to right, var(--primary) {}%, var(--on-primary) 0);",
                    model.brightness_percent()
                ),
            },

            add_controller = gtk::EventControllerScroll {
                set_flags: gtk::EventControllerScrollFlags::BOTH_AXES,
                connect_scroll[sender] => move |_this, _dx, dy| {
                    sender.input(Input::Scroll(dy));
                    false.into()
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
            brightness: 0.,
            _service_handle: service::brightness::Service::forward(
                sender.input_sender().clone(),
                Input::Brightness,
            ),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>) {
        match input {
            Input::Scroll(delta) => {
                let brightness = self.brightness - delta * 0.05;
                if brightness >= 0. && brightness <= 1. {
                    service::brightness::Service::send(
                        service::brightness::Input::SystemBrightness(brightness),
                    );
                }
            }
            Input::Brightness(output) => match output {
                service::brightness::Output::SystemBrightness(brightness) => {
                    self.brightness = brightness
                }
            },
        }
    }
}

impl Model {
    fn brightness_percent(&self) -> i64 {
        (self.brightness * 100.) as i64
    }
}
