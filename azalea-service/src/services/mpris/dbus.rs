pub mod media_player2 {
    use std::collections::HashMap;

    use zbus::proxy;
    use zbus::zvariant::OwnedValue;

    #[proxy(
        interface = "org.mpris.MediaPlayer2.Player",
        default_path = "/org/mpris/MediaPlayer2"
    )]
    pub trait Player {
        fn next(&self) -> zbus::Result<()>;
        fn previous(&self) -> zbus::Result<()>;
        fn pause(&self) -> zbus::Result<()>;
        fn play_pause(&self) -> zbus::Result<()>;
        fn stop(&self) -> zbus::Result<()>;
        fn play(&self) -> zbus::Result<()>;
        fn seek(&self, offset: i64) -> zbus::Result<()>;
        fn set_position(&self, offset: i64) -> zbus::Result<()>;
        fn open_uri(&self, uri: &str) -> zbus::Result<()>;

        #[zbus(property)]
        fn playback_status(&self) -> zbus::fdo::Result<String>;

        #[zbus(property)]
        fn metadata(&self) -> zbus::Result<Metadata>;

        #[zbus(property)]
        fn volume(&self) -> zbus::Result<f64>;

        #[zbus(property(emits_changed_signal = "false"))]
        fn position(&self) -> zbus::Result<i64>;

        #[zbus(signal)]
        fn seeked(&self, position: i64) -> zbus::Result<()>;
    }

    pub type Metadata = HashMap<String, OwnedValue>;
}
