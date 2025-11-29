use azalea_service::StaticHandler;
use gtk::{gio, prelude::*};
use relm4::{
    FactorySender,
    prelude::{DynamicIndex, FactoryComponent},
};

use crate::{
    icon,
    service::{self, search::AppInfo},
};

#[derive(Debug)]
pub struct Model {
    visible: bool,
    app_info: AppInfo,
}

#[derive(Clone, Debug)]
pub enum Input {
    Click,
    Filter(String),
}

#[derive(Debug)]
pub enum Output {}

#[relm4::factory(pub)]
impl FactoryComponent for Model {
    type Init = AppInfo;
    type Input = Input;
    type Output = Output;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            #[watch]
            set_visible: self.visible,
            connect_clicked => Input::Click,

            gtk::Box {
                set_spacing: 12,

                gtk::Image {
                    set_from_gicon: self.app_info.icon
                        .as_ref()
                        .and_then(|i| gio::Icon::deserialize(&i))
                        .as_ref()
                        .unwrap_or(&gio::ThemedIcon::from_names(&[icon::APPS]).upcast::<gio::Icon>())
                },

                gtk::Label {
                    set_label: &self.app_info.display_name,
                }
            }
        }
    }

    fn init_model(
        app_info: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self {
            visible: false,
            app_info,
        }
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            Input::Click => drop(service::search::Service::send(
                service::search::Input::LaunchApplication(self.app_info.id.clone()),
            )),
            Input::Filter(search) => {
                self.visible =
                    search.len() > 0 && self.app_info.name.to_lowercase().starts_with(&search)
            }
        };
    }
}
