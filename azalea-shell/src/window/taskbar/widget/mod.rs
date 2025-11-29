use crate::register_widgets;

pub mod audio;
pub mod bluetooth;
pub mod media;
pub mod network;
pub mod search;
pub mod separator;
pub mod time;

register_widgets!(
    Audio, audio::Model;
    Bluetooth, bluetooth::Model;
    Media, media::Model;
    Network, network::Model;
    Search, search::Model;
    Separator, separator::Model;
    Time, time::Model;
);
