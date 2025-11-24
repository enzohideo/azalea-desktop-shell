mod icons {
    include!(concat!(env!("OUT_DIR"), "/icons.rs"));
}

pub use icons::FIRST as PREVIOUS;
pub use icons::LAST as NEXT;
pub use icons::PAUSE;
pub use icons::PLAY;

pub use icons::BLUETOOTH;
pub use icons::BLUETOOTH_X;
pub use icons::PLUG_CONNECTED_FILLED as PLUG_CONNECTED;
pub use icons::PLUG_DISCONNECTED_FILLED as PLUG_DISCONNECTED;
pub use icons::RADIOWAVES_1 as WIFI_3;
pub use icons::RADIOWAVES_2 as WIFI_2;
pub use icons::RADIOWAVES_3 as WIFI_1;
pub use icons::RADIOWAVES_4 as WIFI_0;
pub use icons::RADIOWAVES_5 as WIFI_X;
pub use icons::RADIOWAVES_DOTS as WIFI_DOTS;
pub use icons::RADIOWAVES_NONE as WIFI_NONE;
pub use icons::RADIOWAVES_QUESTION_MARK as WIFI_QUESTION_MARK;
pub use icons::WAVES_7 as WIFI_SLEEP;

pub fn init() {
    relm4_icons::initialize_icons(icons::GRESOURCE_BYTES, icons::RESOURCE_PREFIX);
}
