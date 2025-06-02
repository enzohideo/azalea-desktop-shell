use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::prelude::BoxExt;
use relm4::{Component, ComponentParts, ComponentSender, component};

use crate::services;

type Time = chrono::DateTime<chrono::Local>;

crate::init! {
    Model {
        time: Time,
        format: String,
        _time_handle: LocalListenerHandle,
    }

    Config {
        format: String,
    }
}

#[derive(Debug)]
pub enum Message {
    Time(Time),
}

#[component(pub)]
impl Component for Model {
    type Init = Init;
    type Input = Message;
    type Output = ();
    type CommandOutput = Message;

    view! {
        gtk::Box{
            set_spacing: 12,

            gtk::Label {
                #[watch]
                set_label: &format!("{}", model.time.format(&model.format))
            },
        },
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
            _time_handle: services::time::Service::filtered_forward_local(
                sender.input_sender().clone(),
                move |event| {
                    use services::time::Output;

                    let format_contains_time = |date_time: chrono::DateTime<chrono::Local>,
                                                time: &str|
                     -> Option<Message> {
                        if format.contains(time) {
                            Some(Message::Time(date_time))
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
            Message::Time(time) => self.time = time,
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Message::Time(time) => self.time = time,
        }
    }
}
