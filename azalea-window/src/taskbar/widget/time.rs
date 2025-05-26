use azalea_service::{ListenerHandle, services};
use gtk::prelude::BoxExt;
use relm4::{Component, ComponentParts, ComponentSender, component};

type Time = chrono::DateTime<chrono::Local>;

crate::init! {
    Model {
        time: Time,
        format: String,
        time_handle: Option<ListenerHandle>,
    }

    Config {
        format: String,
    }

    Services {
        time: services::time::Service,
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
        let mut model = Model {
            format: init.config.format.clone(),
            time: chrono::Local::now(),
            time_handle: None,
        };
        let widgets = view_output!();

        if let Some(time) = init.services.time {
            let format = init.config.format;

            model.time_handle = Some(time.borrow_mut().filtered_forward(
                sender.input_sender().clone(),
                move |event| {
                    use azalea_service::services::time::Output;

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
            ));
        } else {
            sender.command(|out, shutdown| {
                shutdown
                    .register(async move {
                        loop {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            let now = chrono::Local::now();
                            out.send(Self::CommandOutput::Time(now)).unwrap();
                        }
                    })
                    .drop_on_shutdown()
            });
        }

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
