use crate::register_widgets;

pub mod bluetooth;
pub mod media;
pub mod network;
pub mod search;
pub mod time;

register_widgets!(
    Bluetooth, bluetooth::Model;
    Media, media::Model;
    Network, network::Model;
    Search, search::Model;
    Time, time::Model;
);
