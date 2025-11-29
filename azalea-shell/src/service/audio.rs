use alsa::mixer::{SelemChannelId, SelemId};
use tokio::sync::broadcast;

#[derive(azalea_derive::StaticHandler)]
pub struct Service {
    mixer: alsa::Mixer,
    selem_id: SelemId,
    previous_volume: i64,
    interval_duration: std::time::Duration,
}

#[derive(Clone)]
pub struct Init {
    selem_name: String,
    interval_duration: std::time::Duration,
}

impl Default for Init {
    fn default() -> Self {
        Self {
            selem_name: format!("Master"),
            interval_duration: std::time::Duration::from_secs(1),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Input {
    /// Default sink volume
    SystemVolume(f64),
}

#[derive(Clone, Debug)]
pub enum Output {
    /// Default sink volume
    SystemVolume(f64),
}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = Input;
    type Event = ();
    type Output = Output;

    async fn new(
        init: Self::Init,
        _: flume::Sender<Self::Input>,
        _: broadcast::Sender<Self::Output>,
    ) -> Self {
        let mixer = alsa::mixer::Mixer::new("default", false).unwrap();

        Self {
            mixer,
            selem_id: SelemId::new(&init.selem_name, 0),
            interval_duration: init.interval_duration,
            previous_volume: 0,
        }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        _output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::SystemVolume(volume_percent) => {
                if let Some(selem) = self.mixer.find_selem(&self.selem_id) {
                    let (min_volume, max_volume) = selem.get_playback_volume_range();

                    let _ = selem.set_playback_volume_all(
                        (volume_percent * (max_volume - min_volume) as f64) as i64 + min_volume,
                    );
                } else {
                    azalea_log::warning!("[AUDIO]: Failed to find Master selem");
                }
            }
        }
    }

    async fn event_generator(&mut self) -> Self::Event {
        tokio::time::sleep(self.interval_duration).await;
    }

    async fn event_handler(
        &mut self,
        _event: Self::Event,
        output_sender: &tokio::sync::broadcast::Sender<Self::Output>,
    ) -> azalea_service::Result<()> {
        if let Some(selem) = self.mixer.find_selem(&self.selem_id) {
            let mut count = 0;
            let mut acc_volume = 0;

            for channel_id in SelemChannelId::all() {
                if let Ok(channel_volume) = selem.get_playback_volume(*channel_id) {
                    acc_volume += channel_volume;
                    count += 1;
                }
            }

            let volume_int = acc_volume / count;
            let (min_volume, max_volume) = selem.get_playback_volume_range();

            if self.previous_volume != volume_int {
                let volume_percent = volume_int as f64 / ((max_volume - min_volume) as f64);
                output_sender.send(Output::SystemVolume(volume_percent))?;
                self.previous_volume = volume_int;
            }
        }

        Ok(())
    }
}
