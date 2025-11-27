use std::collections::HashMap;

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
#[derive(Default, clap::Parser, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MonitorMatch {
    connector: Option<Id>,
    manufacturer: Option<String>,
    model: Option<String>,
}

impl From<gdk::Monitor> for MonitorMatch {
    fn from(value: gdk::Monitor) -> Self {
        Self {
            connector: value.connector().map(|v| v.to_string()),
            manufacturer: value.manufacturer().map(|v| v.to_string()),
            model: value.model().map(|v| v.to_string()),
        }
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Monitor {
    /// Monitor is determined on-the-fly (good for popups)
    Dynamic,

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

pub fn monitors() -> Vec<gdk::Monitor> {
    let monitors = gdk::Display::default().unwrap().monitors();
    let mut output = vec![];

    for i in 0..monitors.n_items() {
        let Some(monitor): Option<gdk::Monitor> = monitors.item(i).and_downcast() else {
            continue;
        };
        output.push(monitor);
    }

    output
}

pub fn monitors_to_string() -> String {
    let output: Vec<HashMap<&str, String>> = monitors()
        .iter()
        .map(|monitor| {
            HashMap::from([
                (
                    "connector",
                    monitor
                        .connector()
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                ),
                (
                    "description",
                    monitor
                        .description()
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                ),
                (
                    "manufacturer",
                    monitor
                        .manufacturer()
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                ),
                (
                    "model",
                    monitor.model().map(|v| v.to_string()).unwrap_or_default(),
                ),
            ])
        })
        .collect();

    serde_json::to_string_pretty(&output).unwrap()
}

pub fn get_monitor(index: u32) -> Option<gdk::Monitor> {
    gdk::Display::default().and_then(|display| display.monitors().item(index).and_downcast())
}
