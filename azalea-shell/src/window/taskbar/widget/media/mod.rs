use std::collections::HashMap;

use crate::{
    factory::{self, media::player::PlayerName},
    service::{
        self,
        dbus::mpris::{
            self,
            proxy::{PlaybackRate, PlaybackStatus},
        },
    },
};
use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::{
    glib::object::Cast,
    prelude::{BoxExt, ButtonExt, OrientableExt, PopoverExt, WidgetExt},
};
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, RelmWidgetExt, component,
    prelude::FactoryVecDeque,
};

use crate::{component::image, icon};

struct Player {
    status: PlaybackStatus,
    rate: PlaybackRate,
    title: Option<String>,
    artist: Option<String>,
    length: Option<i64>,
    art_url: Option<String>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            status: Default::default(),
            rate: 1.,
            title: Default::default(),
            artist: Default::default(),
            length: Default::default(),
            art_url: Default::default(),
        }
    }
}

crate::init! {
    Model {
        position: f64,
        selected: Option<PlayerName>,
        players: HashMap<PlayerName, Player>,
        art_cover: relm4::Controller<image::Model>,
        menu: FactoryVecDeque<factory::media::player::Model>,
        _event_listener_handle: LocalListenerHandle,
    }

    Config {}
}

#[derive(Debug)]
pub enum Action {
    Previous,
    Next,
    PlayPause,
}

#[derive(Debug)]
pub enum Input {
    Select(PlayerName),
    Event(mpris::Output),
    Action(Action),
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
            set_reveal_child: !model.players.is_empty(),
            set_transition_type: gtk::RevealerTransitionType::Crossfade,
            set_transition_duration: 300,


            gtk::Box{
                set_spacing: 12,

                gtk::MenuButton {
                    set_hexpand: false,
                    set_vexpand: false,
                    set_valign: gtk::Align::Center,

                    set_direction: gtk::ArrowType::Up,

                    #[wrap(Some)]
                    set_popover = &gtk::Popover {
                        set_position: gtk::PositionType::Right,

                        #[local_ref]
                        menu_widget -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 5,
                        },
                    },
                },

                #[local_ref]
                art_cover_widget -> gtk::Widget {},

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_valign: gtk::Align::Center,
                    set_vexpand: true,
                    inline_css: "font-size: 11px;",

                    gtk::Label {
                        set_halign: gtk::Align::Start,

                        #[watch]
                        set_label: &model.title(),
                    },

                    gtk::Label {
                        set_halign: gtk::Align::Start,

                        #[watch]
                        set_label: &model.artist(),
                    },
                },

                gtk::Button {
                    set_vexpand: false,
                    set_valign: gtk::Align::Center,

                    set_icon_name: icon::PREVIOUS,
                    connect_clicked => Input::Action(Action::Previous)
                },

                gtk::Button {
                    set_vexpand: false,
                    set_valign: gtk::Align::Center,

                    #[watch]
                    set_icon_name: if model.is_playing() { icon::PAUSE } else { icon::PLAY },
                    connect_clicked => Input::Action(Action::PlayPause)
                },

                gtk::Button {
                    set_vexpand: false,
                    set_valign: gtk::Align::Center,

                    set_icon_name: icon::NEXT,
                    connect_clicked => Input::Action(Action::Next)
                },

                gtk::Label {
                    inline_css: "font-size: 13px;",

                    #[watch]
                    set_label:
                        &format!(
                            "{}/{}",
                            Self::format_time(model.position as i64),
                            model.length(),
                        )
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {
            selected: None,
            players: Default::default(),
            position: 0.,

            art_cover: image::Model::builder()
                .launch(image::Init {
                    fallback: None,
                    width: None,
                    height: Some(30),
                })
                .detach(),

            menu: FactoryVecDeque::builder()
                .launch(gtk::Box::default())
                .forward(sender.input_sender(), |output| match output {
                    factory::media::player::Output::Select(name) => Input::Select(name),
                }),

            _event_listener_handle: mpris::Service::forward_local(
                sender.input_sender().clone(),
                Input::Event,
            ),
        };

        let cmd_sender = sender.command_sender().clone();
        sender.oneshot_command(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                drop(cmd_sender.send(CommandOutput::PositionDelta(1e6)));
            }
        });

        let menu_widget = model.menu.widget();
        let art_cover_widget: &gtk::Widget = model.art_cover.widget().upcast_ref();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Select(name) => {
                self.selected = Some(name.clone());
                mpris::Service::send(mpris::Input::UpdateMetadata(name));
            }
            Input::Event(output) => {
                if !self.players.contains_key(&output.name) {
                    azalea_log::debug!("[MPRIS]: Player added with name {}", output.name);
                    // TODO: Implement removal from menu
                    self.menu.guard().push_back(output.name.clone());
                    self.players.insert(output.name.clone(), Default::default());
                    // TODO: custom default filters
                    if self.selected.is_none() || output.name.to_lowercase().contains("music") {
                        self.selected = Some(output.name.clone());
                        self.reset();
                    }
                }
                let Some(player) = self.players.get_mut(&output.name) else {
                    return;
                };
                let is_selected = if let Some(selected) = &self.selected {
                    *selected == output.name
                } else {
                    false
                };
                use service::dbus::mpris::Event;
                match output.event {
                    Event::Volume(_) => {}
                    Event::Position(position) => {
                        if is_selected {
                            self.position = position as f64
                        }
                    }
                    Event::Metadata(metadata) => {
                        player.artist = metadata
                            .artist
                            .map(|v| v.first().unwrap_or(&format!("no artist")).to_owned());
                        player.title = metadata.title;
                        player.length = metadata.length;
                        player.art_url = metadata.art_url;
                        self.reset();
                    }
                    Event::PlaybackStatus(playback_status) => player.status = playback_status,
                    Event::PlaybackRate(playback_rate) => player.rate = playback_rate,
                };
            }
            Input::Action(action) => {
                let Some(name) = self.selected.clone() else {
                    return;
                };
                mpris::Service::send(mpris::Input::Action(match action {
                    Action::Previous => mpris::Action::Previous(name),
                    Action::Next => mpris::Action::Next(name),
                    Action::PlayPause => mpris::Action::PlayPause(name),
                }));
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

    fn is_playing(&self) -> bool {
        self.player()
            .map(|p| matches!(p.status, PlaybackStatus::Playing))
            .unwrap_or(false)
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
        drop(match self.player().and_then(|p| p.art_url.as_ref()) {
            Some(url) => self
                .art_cover
                .sender()
                .send(image::Input::LoadImage(url.to_string())),
            None => self.art_cover.sender().send(image::Input::Unload),
        });

        if let Some(name) = &self.selected {
            mpris::Service::send(mpris::Input::UpdatePositionAndRate(name.clone()));
        }
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
