use gtk::glib::object::Cast;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, component};

use crate::component::image;

crate::init! {
    Model {
        image: relm4::Controller<image::Model>,
    }

    Config {
        image: Option<String>,
    }
}

#[derive(Debug)]
pub enum Input {
    Update(String),
}

#[component(pub)]
impl Component for Model {
    type Init = Init;
    type Input = Input;
    type Output = ();
    type CommandOutput = Input;

    view! {
        gtk::Window {
            #[local_ref]
            image_widget -> gtk::Widget {},
        },
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Model {
            image: image::Model::builder()
                .launch(image::Init {
                    fallback: Some(
                        gtk::gdk::Texture::from_bytes(&gtk::glib::Bytes::from_static(
                            include_bytes!("../../../../assets/azalea-wallpaper.png"),
                        ))
                        .unwrap(),
                    ),
                    width: None,
                    height: None,
                })
                .detach(),
        };

        if let Some(image) = init.config.image {
            sender.input(Input::Update(image));
        }

        let image_widget: &gtk::Widget = model.image.widget().upcast_ref();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Input::Update(image) => self.image.emit(image::Input::LoadImage(image)),
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Input::Update(image) => sender.input(Input::Update(image)),
        }
    }
}
