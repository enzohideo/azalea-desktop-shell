use azalea_service::{LocalListenerHandle, StaticHandler};
use relm4::{Component, ComponentParts, ComponentSender, component};

use crate::{
    icon,
    service::dbus::network_manager::{
        self,
        proxy::{NMConnectivityState, NMState},
    },
};

crate::init! {
    Model {
        state: NMState,
        connectivity: NMConnectivityState,
        _nm_handle: LocalListenerHandle,
    }

    Config {}
}

#[derive(Debug)]
pub enum Input {
    NetworkManager(network_manager::Output),
}

#[component(pub)]
impl Component for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Image {
            #[watch]
            set_icon_name: Some(match model.state {
                NMState::NMStateUnknown => icon::WIFI_QUESTION_MARK,
                NMState::NMStateAsleep => icon::WIFI_SLEEP,
                NMState::NMStateDisconnected => icon::WIFI_X,
                NMState::NMStateDisconnecting => icon::WIFI_DOTS,
                NMState::NMStateConnecting => icon::WIFI_DOTS,
                // TODO: Change icon based on connection quality
                NMState::NMStateConnectedLocal => icon::WIFI_3,
                NMState::NMStateConnectedSite => icon::WIFI_3,
                NMState::NMStateConnectedGlobal => icon::WIFI_3,
            }),
        },
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {
            state: Default::default(),
            connectivity: Default::default(),
            _nm_handle: network_manager::Service::forward_local(
                sender.input_sender().clone(),
                Input::NetworkManager,
            ),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        use network_manager::Output;
        match message {
            Input::NetworkManager(nm_msg) => match nm_msg {
                Output::StateChanged(nmstate) => self.state = nmstate,
                Output::ConnectivityChanged(nmconnectivity_state) => {
                    self.connectivity = nmconnectivity_state
                }
            },
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
