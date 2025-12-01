use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, RelmWidgetExt, component};

use crate::service;

type Time = chrono::DateTime<chrono::Local>;

crate::init! {
    Model {
        time: Time,
        format: String,
        _service_handle: LocalListenerHandle,
    }

    Config {
        format: String,
    }
}

#[derive(Debug)]
pub enum Input {
    Time(Time),
}

#[component(pub)]
impl Component for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();
    type CommandOutput = Input;

    view! {
        gtk::MenuButton {
            #[wrap(Some)]
            set_child = &gtk::Label {
                #[watch]
                set_label: &format!("{}", model.time.format(&model.format)),
            },

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                gtk::Box {
                    gtk::Box {
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Center,
                        set_orientation: gtk::Orientation::Vertical,
                        inline_css: "padding: 0 60px 0 60px",

                        gtk::Label {
                            inline_css: "font-size: 50px",
                            set_label: &format!("{}", model.time.format("%H:%M")),
                        },

                        gtk::Label {
                            set_label: &format!("{}", model.time.format("%B %d, %Y")),
                        },
                    },

                    gtk::Calendar { },
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let format = init.config.format;

        let model = Model {
            format: format.clone(),
            time: chrono::Local::now(),
            _service_handle: service::time::Service::filtered_forward_local(
                sender.input_sender().clone(),
                move |event| {
                    use service::time::Output;

                    let format_contains_time =
                        |date_time: chrono::DateTime<chrono::Local>, time: &str| -> Option<Input> {
                            if format.contains(time) {
                                Some(Input::Time(date_time))
                            } else {
                                None
                            }
                        };

                    match event {
                        Output::Second(date_time) => format_contains_time(date_time, "%S"),
                        Output::Minute(date_time) => format_contains_time(date_time, "%M"),
                        Output::Hour(date_time) => format_contains_time(date_time, "%H"),
                    }
                },
            ),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Time(time) => self.time = time,
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Input::Time(time) => self.time = time,
        }
    }
}
