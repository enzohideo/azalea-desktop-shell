use azalea_service::{
    LocalListenerHandle, StaticHandler, dbus::mpris::media_player2::PlaybackStatus, services,
};
use gtk::prelude::BoxExt;
use relm4::{Component, ComponentParts, ComponentSender, component};

// TODO: Stack factory
crate::init! {
    Model {
        status: PlaybackStatus,
        title: Option<String>,
        artist: Option<String>,
        _event_listener_handle: LocalListenerHandle,
    }

    Config {
    }
}

#[derive(Debug)]
pub enum Input {
    Event(services::dbus::mpris::Output),
}

#[component(pub)]
impl Component for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Revealer {
            #[watch]
            set_reveal_child: if let PlaybackStatus::Stopped = model.status { false } else { true },
            set_transition_type: gtk::RevealerTransitionType::Crossfade,
            set_transition_duration: 300,

            gtk::Box{
                set_spacing: 12,

                gtk::Label {
                    #[watch]
                    set_label: &model.title.to_owned().unwrap_or(format!("no title"))
                },

                gtk::Label {
                    #[watch]
                    set_label: &model.artist.to_owned().unwrap_or(format!("no artist"))
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
            status: PlaybackStatus::Stopped,
            title: None,
            artist: None,
            _event_listener_handle: services::dbus::mpris::Service::forward_local(
                sender.input_sender().clone(),
                Input::Event,
            ),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Event(output) => {
                if output.name.to_lowercase().contains("firefox") {
                    use services::dbus::mpris::Event;
                    match output.event {
                        Event::Volume(_) => {}
                        Event::Metadata(metadata) => {
                            self.artist = metadata
                                .artist
                                .map(|v| v.first().unwrap_or(&format!("no artist")).to_owned());
                            self.title = metadata.title;
                        }
                        Event::Position(_) => {}
                        Event::PlaybackStatus(playback_status) => self.status = playback_status,
                    };
                }
            }
        }
    }

    fn update_cmd(
        &mut self,
        _message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
    }
}
