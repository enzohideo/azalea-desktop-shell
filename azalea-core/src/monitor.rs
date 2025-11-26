use gtk::{
    gdk::{
        self,
        prelude::{DisplayExt, MonitorExt},
    },
    gio::prelude::ListModelExt,
    glib::object::CastNone,
};

/// Monitor connector, used to identify a monitor
pub type Id = String;

/// Used to find a specific monitor
#[derive(Default, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MonitorMatch {
    connector: Option<Id>,
    manufacturer: Option<String>,
    model: Option<String>,
}

#[derive(Default, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Monitor {
    /// Monitor is determined on-the-fly (good for popups)
    ///
    /// Dynamic(true): lazy, create when needed
    /// Dynamic(false): non-lazy, create when app starts
    Dynamic(bool),

    /// Stay attached to a single monitor
    Single(MonitorMatch),

    /// Create multiple instances, each attached to a monitor
    Multi(Vec<MonitorMatch>),

    /// Create an instance for each monitor (good for taskbars)
    #[default]
    All,
}

pub fn all() -> Vec<gdk::Monitor> {
    let monitors = gdk::Display::default().unwrap().monitors();

    let mut output_monitors = vec![];

    for i in 0..monitors.n_items() {
        let Some(monitor): Option<gdk::Monitor> = monitors.item(i).and_downcast() else {
            continue;
        };
        output_monitors.push(monitor);
    }

    output_monitors
}

impl MonitorMatch {
    pub fn find_matches(&self) -> Vec<gdk::Monitor> {
        let monitors = gdk::Display::default().unwrap().monitors();

        let mut matched_monitors = vec![];

        for i in 0..monitors.n_items() {
            let Some(monitor): Option<gdk::Monitor> = monitors.item(i).and_downcast() else {
                continue;
            };

            if let Some(connector) = &self.connector {
                if monitor
                    .connector()
                    .map(|v| v.to_string())
                    .unwrap_or(format!(""))
                    != *connector
                {
                    continue;
                }
            }

            if let Some(manufacturer) = &self.manufacturer {
                if monitor
                    .manufacturer()
                    .map(|v| v.to_string())
                    .unwrap_or(format!(""))
                    != *manufacturer
                {
                    continue;
                }
            }

            if let Some(model) = &self.model {
                if monitor
                    .model()
                    .map(|v| v.to_string())
                    .unwrap_or(format!(""))
                    != *model
                {
                    continue;
                }
            }

            matched_monitors.push(monitor);
        }

        return matched_monitors;
    }
}
