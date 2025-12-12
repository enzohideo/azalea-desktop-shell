use std::collections::HashMap;

use zbus::proxy;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

/// NetworkManager root interface
///
/// See: https://networkmanager.dev/docs/api/latest/gdbus-org.freedesktop.NetworkManager.html
/// https://people.freedesktop.org/~lkundrak/nm-docs/gdbus-org.freedesktop.NetworkManager.html
#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager",
    interface = "org.freedesktop.NetworkManager"
)]
pub trait NetworkManager {
    async fn get_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
    fn sleep(&self, sleep: bool) -> zbus::Result<()>;
    fn enable(&self, enable: bool) -> zbus::Result<()>;

    #[zbus(property)]
    fn networking_enabled(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn wireless_enabled(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn wwan_enabled(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn state(&self) -> zbus::Result<NMState>;

    #[zbus(property)]
    fn connectivity(&self) -> zbus::Result<NMConnectivityState>;

    #[zbus(signal)]
    fn properties_changed(&self, properties: HashMap<String, OwnedValue>) -> zbus::Result<()>;
}

/// NMState values indicate the current overall networking state.
///
/// See: https://people.freedesktop.org/~lkundrak/nm-docs/nm-dbus-types.html#NMState
#[derive(
    Default,
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
    #[default]
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
///
/// See: https://people.freedesktop.org/~lkundrak/nm-docs/nm-dbus-types.html#NMConnectivityState
#[derive(
    Default, Clone, Debug, serde_repr::Serialize_repr, serde_repr::Deserialize_repr, OwnedValue,
)]
#[repr(u32)]
#[zvariant(signature = "u")]
pub enum NMConnectivityState {
    #[default]
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

/// NMDeviceState values indicate the device state.
///
/// See: https://people.freedesktop.org/~lkundrak/nm-docs/nm-dbus-types.html#NMDeviceState
#[derive(
    Default, Clone, Debug, serde_repr::Serialize_repr, serde_repr::Deserialize_repr, OwnedValue,
)]
#[repr(u32)]
#[zvariant(signature = "u")]
pub enum NMDeviceState {
    #[default]
    /// The device's state is unknown
    NMDeviceStateUnknown = 0,

    /// The device is recognized, but not managed by networkmanager
    NMDeviceStateUnmanaged = 10,

    /// The device is managed by networkmanager, but is not available for use. reasons may include
    /// the wireless switched off, missing firmware, no ethernet carrier, missing supplicant or
    /// modem manager, etc.
    NMDeviceStateUnavailable = 20,

    /// The device can be activated, but is currently idle and not connected to a network.
    NMDeviceStateDisconnected = 30,

    /// The device is preparing the connection to the network. this may include operations like
    /// changing the mac address, setting physical link properties, and anything else required to
    /// connect to the requested network.
    NMDeviceStatePrepare = 40,

    /// The device is connecting to the requested network. this may include operations like
    /// associating with the wifi ap, dialing the modem, connecting to the remote bluetooth device,
    /// etc.
    NMDeviceStateConfig = 50,

    /// The device requires more information to continue connecting to the requested network. this
    /// includes secrets like wifi passphrases, login passwords, pin codes, etc.
    NMDeviceStateNeedAuth = 60,

    /// The device is requesting ipv4 and/or ipv6 addresses and routing information from the
    /// network.
    NMDeviceStateIpConfig = 70,

    /// The device is checking whether further action is required for the requested network
    /// connection. this may include checking whether only local network access is available,
    /// whether a captive portal is blocking access to the internet, etc.
    NMDeviceStateIpCheck = 80,

    /// The device is waiting for a secondary connection (like a vpn) which must activated before
    /// the device can be activated
    NMDeviceStateSecondaries = 90,

    /// The device has a network connection, either local or global.
    NMDeviceStateActivated = 100,

    /// A disconnection from the current network connection was requested, and The device is
    /// cleaning up resources used for that connection. the network connection may still be valid.
    NMDeviceStateDeactivating = 110,

    /// The device failed to connect to the requested network and is cleaning up the connection
    /// request
    NMDeviceStateFailed = 120,
}

/// NMDeviceType indicates the type of device, e.g.: wifi, ethernet, bluetooth, etc
///
/// See: https://people.freedesktop.org/~lkundrak/nm-docs/nm-dbus-types.html#NMDeviceType
#[derive(
    Default, Clone, Debug, serde_repr::Serialize_repr, serde_repr::Deserialize_repr, OwnedValue,
)]
#[repr(u32)]
#[zvariant(signature = "u")]
pub enum NMDeviceType {
    #[default]
    NMDeviceTypeUnknown = 0,

    NMDeviceTypeGeneric = 14,

    NMDeviceTypeEthernet = 1,

    NMDeviceTypeWifi = 2,

    NMDeviceTypeUnused1 = 3,

    NMDeviceTypeUnused2 = 4,

    NMDeviceTypeBt = 5,

    NMDeviceTypeOlpcMesh = 6,

    NMDeviceTypeWimax = 7,

    NMDeviceTypeModem = 8,

    NMDeviceTypeInfiniband = 9,

    NMDeviceTypeBond = 10,

    NMDeviceTypeVlan = 11,

    NMDeviceTypeAdsl = 12,

    NMDeviceTypeBridge = 13,

    NMDeviceTypeTeam = 15,

    NMDeviceTypeTun = 16,

    NMDeviceTypeIpTunnel = 17,

    NMDeviceTypeMacvlan = 18,

    NMDeviceTypeVxlan = 19,

    NMDeviceTypeVeth = 20,
}

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Device"
)]
pub trait NetworkManagerDevice {
    fn disconnect(&self) -> zbus::Result<()>;

    #[zbus(property)]
    fn state(&self) -> zbus::Result<NMDeviceState>;

    #[zbus(property)]
    fn device_type(&self) -> zbus::Result<NMDeviceType>;

    #[zbus(property)]
    fn interface(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn active_connection(&self) -> zbus::Result<OwnedObjectPath>;
}
