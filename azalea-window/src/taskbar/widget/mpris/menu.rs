use gtk::prelude::ButtonExt;
use relm4::{
    FactorySender,
    prelude::{DynamicIndex, FactoryComponent},
};

use super::PlayerName;

#[derive(Debug)]
pub struct MenuName {
    name: PlayerName,
}

#[derive(Debug)]
pub enum Input {
    Click,
}

#[derive(Debug)]
pub enum Output {
    Select(PlayerName),
}

#[relm4::factory(pub)]
impl FactoryComponent for MenuName {
    type Init = PlayerName;
    type Input = Input;
    type Output = Output;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            set_label: &self.name,
            connect_clicked => Input::Click
        }
    }

    fn init_model(name: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { name }
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            Input::Click => drop(sender.output(Output::Select(self.name.clone()))),
        }
    }
}
