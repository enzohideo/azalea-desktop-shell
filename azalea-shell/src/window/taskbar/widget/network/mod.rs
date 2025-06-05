use azalea_service::{LocalListenerHandle, StaticHandler};
use relm4::{Component, ComponentParts, ComponentSender, component};

use crate::service::dbus::network_manager::{
    self,
    proxy::{NMConnectivityState, NMState},
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
        gtk::Label {
            #[watch]
            set_label: match model.state {
                NMState::NMStateUnknown => "1",
                NMState::NMStateAsleep => "2",
                NMState::NMStateDisconnected => "3",
                NMState::NMStateDisconnecting => "4",
                NMState::NMStateConnecting => "5",
                NMState::NMStateConnectedLocal => "6",
                NMState::NMStateConnectedSite => "7",
                NMState::NMStateConnectedGlobal => "8",
            }
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
