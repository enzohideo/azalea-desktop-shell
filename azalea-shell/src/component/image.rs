use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::sync::OnceLock;

use base64::Engine;
use relm4::gtk::prelude::{FrameExt, WidgetExt};
use relm4::gtk::{gdk, gdk_pixbuf};
use relm4::{Component, ComponentParts, ComponentSender, RelmWidgetExt, component};

pub struct Model {
    fallback: Option<gdk::Texture>,
    image: Option<gdk::Texture>,
    width: Option<i32>,
    height: Option<i32>,
}

pub struct Init {
    pub fallback: Option<gdk::Texture>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Unload,
    LoadImage(String),
    LoadPixbuf(gdk::gdk_pixbuf::Pixbuf),
}

#[derive(Debug)]
pub enum CommandOutput {
    LoadedImage(String, Option<VecDeque<u8>>),
}

#[component(pub)]
impl Component for Model {
    type CommandOutput = CommandOutput;
    type Input = Input;
    type Output = ();
    type Init = Init;

    view! {
        gtk::Frame {
            inline_css: "border-radius: 6px;",

            #[wrap(Some)]
            set_child = if model.image.is_none() && model.fallback.is_none() {
                gtk::Spinner {
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    start: (),
                }
            } else {
                gtk::Picture {
                    #[watch]
                    set_paintable: if model.image.is_some() {
                        model.image.as_ref()
                    } else {
                        model.fallback.as_ref()
                    },
                    set_can_shrink: true,
                }
            },

            set_width_request: model.width.unwrap_or(-1),
            set_height_request: model.height.unwrap_or(-1),
        },
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            fallback: init.fallback,
            image: None,
            width: init.width,
            height: init.height,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match input {
            Input::LoadImage(url) => {
                if let Some(pixbuf) = Self::cache().borrow().get(&url) {
                    azalea_log::info!(
                        "[IMAGE] Loaded image (cache hit): {}...",
                        Self::truncate(&url)
                    );
                    self.set_image(&pixbuf);
                } else {
                    sender.oneshot_command(async move {
                        let image = Self::load_image(&url).await;
                        azalea_log::info!(
                            "[IMAGE] Loaded image (cache miss): {}...",
                            Self::truncate(&url)
                        );
                        CommandOutput::LoadedImage(url, image)
                    });
                }
            }
            Input::LoadPixbuf(pixbuf) => self.set_image(&pixbuf),
            Input::Unload => self.set_spinner(),
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CommandOutput::LoadedImage(url, Some(data)) => {
                let mut pixbuf = gdk_pixbuf::Pixbuf::from_read(data).unwrap();

                if self.height.is_some() || self.width.is_some() {
                    let width = self
                        .width
                        .unwrap_or_else(|| pixbuf.width() * self.height.unwrap() / pixbuf.height());

                    let height = self
                        .height
                        .unwrap_or_else(|| pixbuf.height() * self.width.unwrap() / pixbuf.width());

                    pixbuf = pixbuf
                        .scale_simple(width, height, gdk_pixbuf::InterpType::Hyper)
                        .unwrap();
                }
                self.set_image(&pixbuf);
                Self::cache().borrow_mut().insert(url, pixbuf);
            }
            CommandOutput::LoadedImage(url, None) => {
                azalea_log::warning!("Failed to load image: {url}")
            }
        }
    }
}

impl Model {
    fn set_spinner(&mut self) {
        self.image = None;
    }

    fn set_image(&mut self, pixbuf: &gdk_pixbuf::Pixbuf) {
        self.image = Some(gdk::Texture::for_pixbuf(&pixbuf));
    }

    async fn load_image(url: &str) -> Option<VecDeque<u8>> {
        Some(match url {
            url if url.starts_with("http") => reqwest::get(url)
                .await
                .ok()?
                .bytes()
                .await
                .ok()?
                .into_iter()
                .collect(),
            base64 if base64.starts_with("data:image") => base64
                .split("base64,")
                .collect::<Vec<&str>>()
                .get(1)
                .and_then(|img| base64::engine::general_purpose::STANDARD.decode(img).ok())?
                .into(),
            file => {
                let mut buffer = vec![];
                File::open(file.strip_prefix("file://").unwrap_or(file))
                    .ok()?
                    .read_to_end(&mut buffer)
                    .ok()?;
                buffer.into()
            }
        })
    }

    fn truncate<'a>(url: &'a str) -> &'a str {
        url.char_indices()
            .nth(50)
            .map(|(size, _)| &url[..size])
            .unwrap_or(&url)
    }

    fn cache() -> Rc<RefCell<HashMap<String, gdk_pixbuf::Pixbuf>>> {
        // TODO: Set max capacity, add basic timestamp (updated on every touch) and remove oldest
        // if max capacity reached
        thread_local! {
            static CACHE: OnceLock<Rc<RefCell<HashMap<String, gdk_pixbuf::Pixbuf>>>> = OnceLock::new();
        }

        CACHE.with(|cache| {
            cache
                .get_or_init(move || Rc::new(RefCell::new(Default::default())))
                .clone()
        })
    }
}
