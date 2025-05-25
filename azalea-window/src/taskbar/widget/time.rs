use gtk::prelude::BoxExt;
use relm4::{Component, ComponentParts, ComponentSender, component};

type Time = chrono::DateTime<chrono::Local>;

crate::init! {
    Model {
        time: Time,
    }

    Config {}

    Services {
        time: azalea_service::services::time::Service,
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
                set_label: &format!("{}", model.time.format("%d/%m/%y"))
            },

            gtk::Label {
                #[watch]
                set_label: &format!("{}", model.time.format("%H:%M:%S"))
            },
        },
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {
            time: chrono::Local::now(),
        };
        let widgets = view_output!();

        if let Some(time) = init.services.time {
            // TODO: Add forward_with_filter
            let input = sender.input_sender().clone();
            time.listen(move |out| {
                use azalea_service::services::time::Output;

                drop(match out {
                    // TODO: Only send if format contains S
                    Output::Second(date_time) => input.send(Message::Time(date_time)),
                    // TODO: Only send if format contains M
                    Output::Minute(date_time) => input.send(Message::Time(date_time)),
                    // TODO: Only send if format contains H
                    Output::Hour(date_time) => input.send(Message::Time(date_time)),
                });

                true
            });
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
