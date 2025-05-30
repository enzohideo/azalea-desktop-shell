use std::collections::HashMap;

use zbus::proxy;
use zbus::zvariant::OwnedValue;
use zbus::zvariant::as_value::optional;

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
    fn playback_status(&self) -> zbus::fdo::Result<PlaybackStatus>;

    #[zbus(property)]
    fn rate(&self) -> zbus::fdo::Result<PlaybackRate>;

    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<Metadata>;

    #[zbus(property)]
    fn volume(&self) -> zbus::Result<f64>;

    #[zbus(property(emits_changed_signal = "false"))]
    fn position(&self) -> zbus::Result<i64>;

    #[zbus(signal)]
    fn seeked(&self, position: i64) -> zbus::Result<()>;
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, OwnedValue)]
#[zvariant(signature = "s")]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

pub type PlaybackRate = f64;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, zbus::zvariant::Type)]
#[zvariant(signature = "dict")]
pub struct Metadata {
    #[serde(rename = "mpris:trackid", with = "optional")]
    pub trackid: Option<String>,
    #[serde(rename = "mpris:length", with = "optional")]
    pub length: Option<i64>,
    #[serde(rename = "mpris:artUrl", with = "optional")]
    pub art_url: Option<String>,

    #[serde(rename = "xesam:title", with = "optional")]
    pub title: Option<String>,
    #[serde(rename = "xesam:url", with = "optional")]
    pub url: Option<String>,
    #[serde(rename = "xesam:artist", with = "optional")]
    pub artist: Option<Vec<String>>,
    #[serde(rename = "xesam:album", with = "optional")]
    pub album: Option<String>,
}

// FIXME: OwnedValue macro did not work
impl TryFrom<zbus::zvariant::OwnedValue> for Metadata {
    type Error = zbus::zvariant::Error;
    #[inline]
    fn try_from(value: zbus::zvariant::OwnedValue) -> zbus::zvariant::Result<Self> {
        let mut fields = <HashMap<String, zbus::zvariant::Value>>::try_from(value)?;
        Ok(Self {
            length: fields
                .get("mpris:length")
                .map(|v| v.try_into())
                .transpose()
                .unwrap_or(None),
            art_url: fields
                .get("mpris:artUrl")
                .map(|v| v.try_into())
                .transpose()
                .unwrap_or(None),
            title: fields
                .get("xesam:title")
                .map(|v| v.try_into())
                .transpose()
                .unwrap_or(None),
            url: fields
                .get("xesam:url")
                .map(|v| v.try_into())
                .transpose()
                .unwrap_or(None),
            artist: fields
                .remove("xesam:artist")
                .map(|v| v.try_into())
                .transpose()
                .unwrap_or(None),
            album: fields
                .get("xesam:album")
                .map(|v| v.try_into())
                .transpose()
                .unwrap_or(None),
            trackid: Some(
                TryInto::<zbus::zvariant::ObjectPath>::try_into(
                    fields
                        .remove("mpris:trackid")
                        .ok_or_else(|| zbus::zvariant::Error::IncorrectType)?,
                )?
                .as_str()
                .to_string(),
            ),
        })
    }
}
