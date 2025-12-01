use azalea_service::{LocalListenerHandle, StaticHandler};
use gtk::prelude::*;
use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, SimpleComponent, component,
};

use crate::{component::image, service};

crate::init! {
    Model {
        taskbar_image: relm4::Controller<image::Model>,
        popup_image: relm4::Controller<image::Model>,
        sysinfo: SystemInformation,
        temperature: (f64, String),
        _service_handle: LocalListenerHandle,
    }

    Config {
    }
}

// TODO: Service that polls for updates
struct SystemInformation {
    os_name: String,
    username: String,
    hostname: String,
    kernel: String,
    cpu: String,
    gpu: String,
    memory: String,
    compositor: String,
}

impl Default for SystemInformation {
    fn default() -> Self {
        Self {
            os_name: ffetch::get_os_name().unwrap_or("unknown".to_string()),
            kernel: ffetch::get_kernel_version().unwrap_or("unknown".to_string()),
            username: ffetch::get_username(),
            hostname: ffetch::get_hostname().unwrap_or("unknown".to_string()),
            cpu: ffetch::get_cpu_name().unwrap_or("unknown".to_string()),
            gpu: ffetch::get_gpu(),
            memory: ffetch::get_memory().unwrap_or("unknown".to_string()),
            compositor: ffetch::get_desktop_env(),
        }
    }
}

#[derive(Debug)]
pub enum Input {
    Weather(service::weather::Output),
}

#[derive(Debug)]
pub enum CommandOutput {}

#[component(pub)]
impl SimpleComponent for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();

    view! {
        gtk::MenuButton {
            #[wrap(Some)]
            set_child= &gtk::Box {
                set_spacing: 12,

                #[local_ref]
                taskbar_image_widget -> gtk::Widget {
                },

                gtk::Label {
                    set_label: "Azalea"
                },
            },

            #[wrap(Some)]
            set_popover = &gtk::Popover {
                gtk::Box {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        #[local_ref]
                        popup_image_widget -> gtk::Widget {
                        },

                        gtk::Separator {},

                        gtk::Box {
                            set_spacing: 8,
                            set_halign: gtk::Align::Start,
                            gtk::Label {
                                set_css_classes: &[ "azalea-primary-fg", ],
                                set_label: &format!("{}@{}", model.sysinfo.username, model.sysinfo.hostname),
                            },
                        },

                        gtk::Box {
                            set_spacing: 8,
                            set_halign: gtk::Align::Start,
                            gtk::Label {
                                set_css_classes: &[ "azalea-secondary-fg", ],
                                set_label: " OS:",
                            },
                            gtk::Label {
                                set_label: &model.sysinfo.os_name,
                            },
                        },

                        gtk::Box {
                            set_spacing: 8,
                            set_halign: gtk::Align::Start,
                            gtk::Label {
                                set_css_classes: &[ "azalea-tertiary-fg", ],
                                set_label: " Kernel:",
                            },
                            gtk::Label {
                                set_label: &model.sysinfo.kernel,
                            },
                        },

                        gtk::Box {
                            set_spacing: 8,
                            set_halign: gtk::Align::Start,
                            gtk::Label {
                                set_css_classes: &[ "azalea-primary-fg", ],
                                set_label: "󰻠 CPU:",
                            },
                            gtk::Label {
                                set_label: &model.sysinfo.cpu,
                            },
                        },

                        gtk::Box {
                            set_spacing: 8,
                            set_halign: gtk::Align::Start,
                            gtk::Label {
                                set_css_classes: &[ "azalea-secondary-fg", ],
                                set_label: "󰍛 GPU:",
                            },
                            gtk::Label {
                                set_label: &model.sysinfo.gpu,
                            },
                        },

                        gtk::Box {
                            set_spacing: 8,
                            set_halign: gtk::Align::Start,
                            gtk::Label {
                                set_css_classes: &[ "azalea-tertiary-fg", ],
                                set_label: "󰑭 Memory:",
                            },
                            gtk::Label {
                                set_label: &model.sysinfo.memory,
                            },
                        },

                        gtk::Box {
                            set_spacing: 8,
                            set_halign: gtk::Align::Start,
                            gtk::Label {
                                set_css_classes: &[ "azalea-secondary-fg", ],
                                set_label: " Compositor:",
                            },
                            gtk::Label {
                                set_label: &model.sysinfo.compositor,
                            },
                        },
                    },
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {
            temperature: Default::default(),
            sysinfo: Default::default(),
            taskbar_image: image::Model::builder()
                .launch(image::Init {
                    fallback: None,
                    width: None,
                    height: Some(30),
                })
                .detach(),
            popup_image: image::Model::builder()
                .launch(image::Init {
                    fallback: None,
                    width: None,
                    height: Some(200),
                })
                .detach(),
            _service_handle: service::weather::Service::forward_local(
                sender.input_sender().clone(),
                Input::Weather,
            ),
        };

        drop(model.taskbar_image.sender().send(image::Input::LoadBytes(
            include_bytes!("../../../../../../assets/azalea-logo.png").to_vec(),
        )));

        drop(model.popup_image.sender().send(image::Input::LoadBytes(
            include_bytes!("../../../../../../assets/azalea-logo.png").to_vec(),
        )));

        let taskbar_image_widget: &gtk::Widget = model.taskbar_image.widget().upcast_ref();
        let popup_image_widget: &gtk::Widget = model.popup_image.widget().upcast_ref();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Input::Weather(weather) => match weather {
                service::weather::Output::Temperature(temperature) => {
                    self.temperature = temperature;
                }
            },
        }
    }
}
