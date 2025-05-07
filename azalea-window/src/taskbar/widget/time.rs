use azalea_service::FromServices;
use gtk::prelude::BoxExt;
use relm4::{Component, ComponentParts, ComponentSender, component};

type Time = chrono::DateTime<chrono::Local>;

pub struct Model {
    time: Time,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Message {
    Time(Time),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {}

pub struct Services {
    time: Option<std::rc::Rc<azalea_service::Service<azalea_service::time::Model>>>,
}

impl crate::InitExt for Model {
    type Config = Config;
    type Services = Services;
}

impl<ParentServices> FromServices<ParentServices> for Services
where
    ParentServices: azalea_service::HasService<azalea_service::time::Model>,
{
    fn inherit(value: &ParentServices) -> Self {
        Self {
            time: value.get_service(),
        }
    }
}

#[component(pub)]
impl Component for Model {
    type Init = crate::Init<Self>;
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
            time.forward(sender.input_sender().clone(), Message::Time);
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
