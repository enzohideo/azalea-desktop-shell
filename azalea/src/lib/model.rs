use bincode::{Decode, Encode};
use clap::Parser;

pub mod layer_shell {
    pub type Namespace = String;

    pub enum Layer {
        Background,
        Bottom,
        Top,
        Overlay,
    }
}

#[derive(Parser, Encode, Decode, Debug)]
pub struct Window {
    pub id: String,
    pub init: (),
}
