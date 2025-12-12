use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::prelude::*;
use relm4::{
    Component, ComponentParts, ComponentSender, RelmWidgetExt, component, prelude::FactoryVecDeque,
};

use crate::{
    factory, icon,
    service::dbus::network_manager::{
        self,
        proxy::{NMConnectivityState, NMState},
    },
};

crate::init! {
    Model {
        enabled: bool,
        state_connectivity: (NMState, NMConnectivityState),
        _nm_handle: LocalListenerHandle,
        devices_menu: FactoryVecDeque<factory::network::device::Model>,
    }

    Config {}
}

#[derive(Debug)]
pub enum Input {
    Enable(bool),
    NetworkManager(network_manager::Output),
}

#[component(pub)]
impl Component for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::MenuButton {
            set_hexpand: false,
            set_vexpand: false,
            set_valign: gtk::Align::Center,

            #[watch]
            set_icon_name: match model.state_connectivity {
                (NMState::NMStateUnknown        , _) => icon::WIFI_QUESTION_MARK,
                (NMState::NMStateAsleep         , _)=> icon::WIFI_SLEEP,
                (NMState::NMStateDisconnected   , _)=> icon::WIFI_X,
                (NMState::NMStateDisconnecting  , _)=> icon::WIFI_DOTS,
                (NMState::NMStateConnecting     , _)=> icon::WIFI_DOTS,
                (NMState::NMStateConnectedLocal , _)=> icon::WIFI_NONE,
                (NMState::NMStateConnectedSite  , _)=> icon::WIFI_NONE,
                (NMState::NMStateConnectedGlobal, NMConnectivityState::NMConnectivityFull)=> icon::WIFI_3,
                (NMState::NMStateConnectedGlobal, NMConnectivityState::NMConnectivityLimited)=> icon::WIFI_2,
                (NMState::NMStateConnectedGlobal, _)=> icon::WIFI_0,
            },

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                set_position: gtk::PositionType::Right,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Box {
                        set_spacing: 12,

                        gtk::Label::new(Some("Network")) {
                            inline_css: r#"
                                font-weight: bold;
                            "#,

                            #[watch]
                            set_css_classes: if model.enabled {
                                &[ "azalea-primary-fg" ]
                            } else {
                                &[]
                            },
                            set_halign: gtk::Align::Start,
                            set_hexpand: true,
                        },

                        gtk::Switch {
                            set_halign: gtk::Align::End,

                            #[watch]
                            #[block_signal(toggle_state)]
                            set_active: model.enabled,

                            connect_state_set[sender] => move |_, on| {
                                sender.input(Input::Enable(on));
                                false.into()
                            } @toggle_state,
                        },
                    },

                    gtk::Separator {},

                    gtk::Label::new(Some("Devices")) {
                        set_css_classes: &[ "azalea-primary-fg" ],
                        set_halign: gtk::Align::Start,
                        set_hexpand: true,
                    },

                    #[local_ref]
                    devices_widget -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,
                    }
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
            enabled: true,
            state_connectivity: Default::default(),
            _nm_handle: network_manager::Service::forward_local(
                sender.input_sender().clone(),
                Input::NetworkManager,
            ),
            devices_menu: FactoryVecDeque::builder()
                .launch(gtk::Box::default())
                .detach(),
        };

        network_manager::Service::send(network_manager::Input::Update);

        let devices_widget = model.devices_menu.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        use network_manager::Output;
        match message {
            Input::NetworkManager(nm_msg) => match nm_msg {
                Output::NetworkingEnabledChanged(on) => self.enabled = on,
                Output::StateChanged(nmstate) => self.state_connectivity.0 = nmstate,
                Output::ConnectivityChanged(nmconnectivity_state) => {
                    self.state_connectivity.1 = nmconnectivity_state
                }
                Output::Devices(devices) => {
                    let mut guard = self.devices_menu.guard();

                    guard.clear();

                    for device in devices {
                        guard.push_back(device);
                    }
                }
            },
            Input::Enable(on) => {
                network_manager::Service::send(network_manager::Input::Enable(on));
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
