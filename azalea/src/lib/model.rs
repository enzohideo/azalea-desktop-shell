use bincode::{Decode, Encode};
use clap::Parser;

#[derive(Parser, Encode, Decode, Debug)]
pub struct Window {
    pub namespace: String,
    pub init: (), // # TODO: Make it generic
}
