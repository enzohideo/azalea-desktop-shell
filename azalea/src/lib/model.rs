use bincode::{Decode, Encode};
use clap::Parser;

#[derive(Parser, Encode, Decode, Debug)]
pub struct Window {
    pub title: String,
    pub init: (), // # TODO: Make it generic
}
