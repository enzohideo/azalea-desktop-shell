use std::collections::HashMap;

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

#[derive(Default)]
struct Player {
    status: PlaybackStatus,
    rate: PlaybackRate,
    title: Option<String>,
    artist: Option<String>,
    length: Option<i64>,
}

type PlayerName = String;

// TODO: Stack factory
crate::init! {
    Model {
        position: f64,
        selected: Option<PlayerName>,
        players: HashMap<PlayerName, Player>,
        art_cover: relm4::Controller<image::Model>,
        _event_listener_handle: LocalListenerHandle,
    }

    Config {}
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
            set_reveal_child: model
                .player()
                .map(|p| match p.status {
                    PlaybackStatus::Stopped => false,
                    _ => true
                })
                .unwrap_or(false),
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
                        set_label: &model.title(),
                    },

                    gtk::Box{
                        set_spacing: 12,

                        gtk::Label {
                            #[watch]
                            set_label: &model.artist(),
                        },

                        gtk::Label {
                            #[watch]
                            set_label:
                                &format!(
                                    "{}/{}",
                                    Self::format_time(model.position as i64),
                                    model.length(),
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
            selected: None,
            players: Default::default(),
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
                if !self.players.contains_key(&output.name) {
                    azalea_log::debug!("[MPRIS]: Player added with name {}", output.name);
                    self.players.insert(output.name.clone(), Default::default());
                    // TODO: custom default filters
                    if output.name.to_lowercase().contains("music") {
                        self.selected = Some(output.name.clone());
                        self.reset();
                    }
                }
                let Some(player) = self.players.get_mut(&output.name) else {
                    return;
                };
                use services::dbus::mpris::Event;
                match output.event {
                    Event::Volume(_) => {}
                    Event::Metadata(metadata) => {
                        player.artist = metadata
                            .artist
                            .map(|v| v.first().unwrap_or(&format!("no artist")).to_owned());
                        player.title = metadata.title;
                        player.length = metadata.length;
                        drop(match metadata.art_url {
                            Some(url) => self.art_cover.sender().send(image::Input::LoadImage(url)),
                            None => self.art_cover.sender().send(image::Input::Unload),
                        });
                        self.reset();
                    }
                    Event::PlaybackStatus(playback_status) => player.status = playback_status,
                    Event::PlaybackRate(playback_rate) => player.rate = playback_rate,
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
        let Some(player) = self.player_mut() else {
            return;
        };
        match message {
            CommandOutput::PositionDelta(delta) => {
                if let PlaybackStatus::Playing = player.status {
                    self.position += delta * player.rate
                }
            }
        }
    }
}

impl Model {
    fn player(&self) -> Option<&Player> {
        self.players.get(self.selected.as_ref()?)
    }

    fn player_mut(&mut self) -> Option<&mut Player> {
        self.players.get_mut(self.selected.as_ref()?)
    }

    fn title(&self) -> String {
        self.player()
            .and_then(|p| p.title.to_owned())
            .unwrap_or(format!("no title"))
    }

    fn artist(&self) -> String {
        self.player()
            .and_then(|p| p.artist.to_owned())
            .unwrap_or(format!("no artist"))
    }

    fn length(&self) -> String {
        self.player()
            .and_then(|p| p.length.map(Self::format_time))
            .unwrap_or(format!("00:00"))
    }

    fn reset(&mut self) {
        self.position = 0.;
    }

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
