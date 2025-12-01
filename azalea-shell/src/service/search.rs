use std::{collections::HashMap, path::PathBuf};

use gtk::{
    gio::{self, prelude::*},
    glib,
};
use tokio::sync::broadcast;

#[derive(Default, azalea_derive::StaticHandler)]

pub struct Service {
    applications: HashMap<AppId, AppInfo>,
}

pub type AppId = String;

#[derive(Clone, Debug)]
pub struct AppInfo {
    pub id: AppId,
    pub name: String,
    pub icon: Option<glib::Variant>,
    pub display_name: String,
    pub executable: PathBuf,
    pub command: PathBuf,
}

impl From<&gio::AppInfo> for AppInfo {
    fn from(value: &gio::AppInfo) -> Self {
        Self {
            id: value
                .id()
                .map(|id| id.to_string())
                .unwrap_or(value.name().to_string()),
            name: value.name().to_string(),
            icon: value.icon().and_then(|icon| icon.serialize()),
            display_name: value.display_name().to_string(),
            executable: value.executable(),
            command: value.commandline().unwrap_or(value.executable()),
        }
    }
}

#[derive(Default, Clone)]
pub struct Init {}

#[derive(Clone, Debug)]
pub enum Input {
    LaunchApplication(AppId),

    /// Generic search
    Search(String),

    /// Search only for applications
    SearchApplication(String),

    /// Get all applications in case you want to search "locally"
    GetAllApplications(flume::Sender<Vec<AppInfo>>),
}

#[derive(Clone, Debug)]
pub enum Output {
    Applications(Vec<AppInfo>),
}

impl azalea_service::Service for Service {
    type Init = Init;
    type Input = Input;
    type Event = ();
    type Output = Output;
    const DISABLE_EVENTS: bool = true;

    async fn new(
        _init: Self::Init,
        _: flume::Sender<Self::Input>,
        _: broadcast::Sender<Self::Output>,
    ) -> Self {
        Self {
            applications: gio::AppInfo::all()
                .into_iter()
                .map(|app| {
                    let app = AppInfo::from(&app);
                    (app.id.clone(), app)
                })
                .collect(),
        }
    }

    async fn message(
        &mut self,
        input: Self::Input,
        output_sender: &broadcast::Sender<Self::Output>,
    ) {
        match input {
            Input::Search(term) | Input::SearchApplication(term) => {
                drop(
                    output_sender.send(Output::Applications(
                        self.applications
                            .values()
                            .filter(|app| app.name.starts_with(&term))
                            .map(|app| app.to_owned())
                            .collect(),
                    )),
                );
            }
            Input::GetAllApplications(sender) => {
                drop(
                    sender.send(
                        self.applications
                            .values()
                            .map(|app| app.to_owned())
                            .collect(),
                    ),
                );
            }
            Input::LaunchApplication(app_id) => {
                let Some(app) = self.applications.get(&app_id) else {
                    azalea_log::warning!("Application not found: {app_id}");
                    return;
                };
                match std::process::Command::new(&app.executable)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                {
                    Ok(_) => azalea_log::debug!("Launched application: {:?}", app.executable),
                    Err(e) => azalea_log::warning!(
                        "Failed to launch application {:?}: {e}",
                        app.executable
                    ),
                }
            }
        }
    }
}
