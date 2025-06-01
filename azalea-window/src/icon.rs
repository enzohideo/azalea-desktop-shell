mod icons {
    include!(concat!(env!("OUT_DIR"), "/icons.rs"));
}

pub use icons::FIRST as PREVIOUS;
pub use icons::LAST as NEXT;
pub use icons::PAUSE;
pub use icons::PLAY;

pub fn init() {
    relm4_icons::initialize_icons(icons::GRESOURCE_BYTES, icons::RESOURCE_PREFIX);
}
