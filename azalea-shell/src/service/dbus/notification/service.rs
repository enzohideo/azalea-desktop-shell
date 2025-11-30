use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
}

pub enum Event {
    Notify(Notification),
}

/// org.freedesktop.Notifications service state
pub struct Notifications {
    last_id_used: u32,
    tx: flume::Sender<Event>,
}

impl Notifications {
    pub fn new(tx: flume::Sender<Event>) -> Notifications {
        Notifications {
            last_id_used: 0,
            tx,
        }
    }
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl Notifications {
    fn get_server_information(&self) -> zbus::fdo::Result<(String, String, String, String)> {
        Ok((
            format!(env!("CARGO_PKG_NAME")),
            format!(env!("CARGO_PKG_AUTHORS")),
            format!(env!("CARGO_PKG_VERSION")),
            format!("1.2"),
        ))
    }

    fn get_capabilities(&self) -> zbus::fdo::Result<Vec<String>> {
        Ok(vec!["body", "actions", "icon-static"]
            .into_iter()
            .map(|s| s.to_string())
            .collect())
    }

    async fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        _actions: Vec<String>,
        _hints: HashMap<String, zbus::zvariant::Value<'_>>,
        _expire_timeout: i32,
    ) -> u32 {
        let id = if replaces_id != 0 {
            replaces_id
        } else {
            self.last_id_used = self.last_id_used + 1;
            self.last_id_used
        };

        azalea_log::debug!(
            "[NOTIFICATIONS]: Received notification: {app_name} {app_icon} {summary} {body}"
        );

        if let Err(e) = self
            .tx
            .send_async(Event::Notify(Notification {
                id,
                app_name,
                app_icon,
                summary,
                body,
            }))
            .await
        {
            azalea_log::warning!(
                "[NOTIFICATIONS]: Failed to send notification from zbus to main service: {e}"
            );
        };

        id
    }
}
