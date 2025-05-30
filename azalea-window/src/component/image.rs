use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;

use relm4::gtk::prelude::{FrameExt, WidgetExt};
use relm4::gtk::{gdk, gdk_pixbuf};
use relm4::{Component, ComponentParts, ComponentSender, RelmWidgetExt};

#[derive(Debug, Clone)]
pub struct Model {
    // TODO: Shared lazy static cache
    // thread_local! {
    //     static CACHE: OnceLock<Rc<RefCell<HashMap<String, gdk_pixbuf::Pixbuf>>>> = OnceLock::new();
    // }
    cache: HashMap<String, gdk_pixbuf::Pixbuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Unload,
    LoadImage(String),
}

#[derive(Debug)]
pub enum CommandOutput {
    LoadedImage(String, Option<VecDeque<u8>>),
}

impl Component for Model {
    type CommandOutput = CommandOutput;
    type Input = Input;
    type Output = ();
    type Init = ();
    type Root = gtk::Frame;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk::Frame::default()
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widget = gtk::Box::default();
        root.set_child(Some(&widget));

        let model = Self {
            cache: Default::default(),
        };
        model.set_spinner(&root);

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match input {
            Input::LoadImage(url) => {
                sender.oneshot_command(async move {
                    let image = Self::load_image(&url).await;
                    CommandOutput::LoadedImage(url, image)
                });
            }
            Input::Unload => self.set_spinner(root),
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            CommandOutput::LoadedImage(key, Some(data)) => {
                let pixbuf = gdk_pixbuf::Pixbuf::from_read(data).unwrap();
                self.set_image(root, &pixbuf);
                self.cache.insert(key, pixbuf);
            }
            CommandOutput::LoadedImage(_key, None) => todo!(),
        }
    }
}

impl Model {
    fn set_spinner(&self, root: &<Self as Component>::Root) {
        relm4::view! {
            #[local_ref]
            root -> gtk::Frame {
                #[name(spinner)]
                gtk::Spinner {
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    start: (),
                }
            }
        }
    }

    fn set_image(&self, root: &<Self as Component>::Root, pixbuf: &gdk_pixbuf::Pixbuf) {
        relm4::view! {
            #[local_ref]
            root -> gtk::Frame {
                inline_css: "border-radius: 6px;",
                gtk::Picture::for_paintable(&gdk::Texture::for_pixbuf(&pixbuf)) {}
            }
        }
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
            file => {
                let mut buffer = vec![];
                File::open(file.strip_prefix("file://")?)
                    .ok()?
                    .read_to_end(&mut buffer)
                    .ok()?;
                buffer.into()
            }
        })
    }
}
