use alsa::mixer::{SelemChannelId, SelemId};

fn main() {
    let mixer = alsa::mixer::Mixer::new("default", false).unwrap();

    let selem_id = SelemId::new("Master", 0);
    let selem = mixer
        .find_selem(&selem_id)
        .ok_or("Master control not found")
        .unwrap();

    let (min_volume, max_volume) = selem.get_playback_volume_range();

    let _ = selem.set_playback_volume_all(
        (50. / 100. * (max_volume - min_volume) as f64) as i64 + min_volume,
    );

    for channel_id in &[
        SelemChannelId::FrontLeft,
        SelemChannelId::FrontRight,
        SelemChannelId::RearLeft,
        SelemChannelId::RearRight,
    ] {
        if let Ok(volume) = selem.get_playback_volume(*channel_id) {
            println!("- Channel {:?}, volume {}", channel_id, volume);
        }
    }
}
