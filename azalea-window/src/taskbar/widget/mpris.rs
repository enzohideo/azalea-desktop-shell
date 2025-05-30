use azalea_service::{
    LocalListenerHandle, StaticHandler,
    dbus::mpris::media_player2::{PlaybackRate, PlaybackStatus},
    services,
};
use gtk::{
    glib::object::Cast,
    prelude::{BoxExt, OrientableExt},
};
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, component};

use crate::component::image;

// TODO: Stack factory
crate::init! {
    Model {
        status: PlaybackStatus,
        rate: PlaybackRate,
        title: Option<String>,
        artist: Option<String>,
        length: Option<i64>,
        position: f64,

        art_cover: relm4::Controller<image::Model>,

        _event_listener_handle: LocalListenerHandle,
    }

    Config {
    }
}

#[derive(Debug)]
pub enum Input {
    Event(services::dbus::mpris::Output),
}

#[derive(Debug)]
pub enum CommandOutput {
    PositionDelta(f64),
}

#[component(pub)]
impl Component for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();
    type CommandOutput = CommandOutput;

    view! {
        gtk::Revealer {
            #[watch]
            set_reveal_child: if let PlaybackStatus::Stopped = model.status { false } else { true },
            set_transition_type: gtk::RevealerTransitionType::Crossfade,
            set_transition_duration: 300,

            gtk::Box{
                set_spacing: 12,

                #[local_ref]
                art_cover_widget -> gtk::Widget {},

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Label {
                        #[watch]
                        set_label: &model.title.to_owned().unwrap_or(format!("no title"))
                    },

                    gtk::Box{
                        set_spacing: 12,

                        gtk::Label {
                            #[watch]
                            set_label: &model.artist.to_owned().unwrap_or(format!("no artist"))
                        },

                        gtk::Label {
                            #[watch]
                            set_label:
                                &format!(
                                    "{}/{}",
                                    Self::format_time(model.position as i64),
                                    model.length.map(Self::format_time).unwrap_or(format!("00:00"))
                                )
                        },
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let art_cover = image::Model::builder().launch(()).detach();

        let model = Model {
            status: PlaybackStatus::Stopped,
            rate: 1.,
            title: None,
            artist: None,
            length: None,
            position: 0.,

            art_cover,
            _event_listener_handle: services::dbus::mpris::Service::forward_local(
                sender.input_sender().clone(),
                Input::Event,
            ),
        };

        let cmd_sender = sender.command_sender().clone();
        sender.oneshot_command(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                drop(cmd_sender.send(CommandOutput::PositionDelta(1500000.)));
            }
        });

        let art_cover_widget: &gtk::Widget = model.art_cover.widget().upcast_ref();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Event(output) => {
                use services::dbus::mpris::Event;
                match output.event {
                    Event::Volume(_) => {}
                    Event::Metadata(metadata) => {
                        self.artist = metadata
                            .artist
                            .map(|v| v.first().unwrap_or(&format!("no artist")).to_owned());
                        self.title = metadata.title;
                        drop(match metadata.art_url {
                            Some(url) => self.art_cover.sender().send(image::Input::LoadImage(url)),
                            None => self.art_cover.sender().send(image::Input::Unload),
                        });
                        self.length = metadata.length;
                        self.position = 0.;
                    }
                    Event::PlaybackStatus(playback_status) => self.status = playback_status,
                    Event::PlaybackRate(playback_rate) => self.rate = playback_rate,
                };
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CommandOutput::PositionDelta(delta) => {
                if let PlaybackStatus::Playing = self.status {
                    self.position += delta * self.rate
                }
            }
        }
    }
}

impl Model {
    fn format_time(us: i64) -> String {
        let time = us / 1000000;
        let hours = time / 3600;
        let minutes = time / 60 - hours * 60;
        let seconds = time - minutes * 60 - hours * 3600;

        if hours == 0 {
            format!("{:0>2}:{:0>2}", minutes, seconds)
        } else {
            format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)
        }
    }
}
