use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::prelude::*;
use relm4::RelmWidgetExt;
use relm4::{ComponentParts, ComponentSender, SimpleComponent, component};

use crate::icon;

use crate::service;

crate::init! {
    Model {
        system_volume: f64,
        _service_handle: LocalListenerHandle,
    }

    Config {
    }
}

#[derive(Debug)]
pub enum Input {
    Scroll(f64),
    Audio(service::audio::Output),
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
                set_icon_name: Some(icon::AUDIO),
            },

            gtk::Label {
                #[watch]
                set_label: &format!("{}%", model.system_volume_percent()),
            },

            gtk::Frame {
                set_width_request: 100,
                set_height_request: 5,
                set_vexpand: false,
                set_valign: gtk::Align::Center,

                #[watch]
                inline_css: &format!(
                    "background-image: linear-gradient(to right, var(--primary) {}%, var(--on-primary) 0);",
                    model.system_volume_percent()
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
            system_volume: 0.,
            _service_handle: service::audio::Service::forward_local(
                sender.input_sender().clone(),
                Input::Audio,
            ),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>) {
        match input {
            Input::Scroll(delta) => {
                let volume = self.system_volume - delta * 0.01;
                if volume >= 0. && volume <= 1. {
                    self.system_volume = volume;
                    service::audio::Service::send(service::audio::Input::SystemVolume(
                        self.system_volume,
                    ));
                }
            }
            Input::Audio(output) => match output {
                service::audio::Output::SystemVolume(volume) => self.system_volume = volume,
            },
        }
    }
}

impl Model {
    fn system_volume_percent(&self) -> i64 {
        (self.system_volume * 100.) as i64
    }
}
