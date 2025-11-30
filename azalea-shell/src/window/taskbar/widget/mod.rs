use crate::register_widgets;

pub mod audio;
pub mod bluetooth;
pub mod brightness;
pub mod media;
pub mod network;
pub mod notification;
pub mod search;
pub mod separator;
pub mod time;

register_widgets!(
    Audio, audio::Model;
    Brightness, brightness::Model;
    Bluetooth, bluetooth::Model;
    Media, media::Model;
    Network, network::Model;
    Notification, notification::Model;
    Search, search::Model;
    Separator, separator::Model;
    Time, time::Model;
);
