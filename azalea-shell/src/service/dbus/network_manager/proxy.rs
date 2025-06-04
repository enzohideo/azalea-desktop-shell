use std::collections::HashMap;

use zbus::proxy;
use zbus::zvariant::OwnedValue;

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager",
    interface = "org.freedesktop.NetworkManager"
)]
pub trait NetworkManager {
    fn sleep(&self, sleep: bool) -> zbus::Result<()>;
    fn enable(&self, enable: bool) -> zbus::Result<()>;

    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn state(&self) -> zbus::Result<NMState>;

    #[zbus(property)]
    fn connectivity(&self) -> zbus::Result<NMConnectivityState>;

    #[zbus(signal)]
    fn properties_changed(&self, properties: HashMap<String, OwnedValue>) -> zbus::Result<()>;

    // TODO: Check if this is necessary (or if state sends the default propertychanged signal)
    // #[zbus(signal)]
    // fn state_changed(&self, state: NMState) -> zbus::Result<()>;
}

/// NMState values indicate the current overall networking state.
#[derive(
    Clone,
    Debug,
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
    OwnedValue,
    zbus::zvariant::Type,
)]
#[repr(u32)]
#[zvariant(signature = "u")]
pub enum NMState {
    /// Networking state is unknown
    NMStateUnknown = 0,

    /// Networking is not enabled
    NMStateAsleep = 10,

    /// There is no active network connection
    NMStateDisconnected = 20,

    /// Network connections are being cleaned up
    NMStateDisconnecting = 30,

    /// A network connection is being started
    NMStateConnecting = 40,

    /// There is only local IPv4 and/or IPv6 connectivity
    NMStateConnectedLocal = 50,

    /// There is only site-wide IPv4 and/or IPv6 connectivity
    NMStateConnectedSite = 60,

    /// There is global IPv4 and/or IPv6 Internet connectivity
    NMStateConnectedGlobal = 70,
}

/// NMState values indicate the current overall networking state.
#[derive(Clone, Debug, serde_repr::Serialize_repr, serde_repr::Deserialize_repr, OwnedValue)]
#[repr(u32)]
#[zvariant(signature = "u")]
pub enum NMConnectivityState {
    /// Network connectivity is unknown.
    NMConnectivityUnknown = 1,

    /// The host is not connected to any network.
    NMConnectivityNone = 2,

    /// The host is behind a captive portal and cannot reach the full Internet.
    NMConnectivityPortal = 3,

    /// The host is connected to a network, but does not appear to be able to reach the full Internet.
    NMConnectivityLimited = 4,

    /// The host is connected to a network, and appears to be able to reach the full Internet.
    NMConnectivityFull = 5,
}
