use std::collections::HashMap;

pub mod layer_shell {
    use clap::Parser;

    pub type Namespace = String;

    #[derive(clap::ValueEnum, serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub enum Layer {
        Background,
        Bottom,
        Top,
        Overlay,
    }

    impl Into<gtk4_layer_shell::Layer> for &Layer {
        fn into(self) -> gtk4_layer_shell::Layer {
            match self {
                Layer::Background => gtk4_layer_shell::Layer::Background,
                Layer::Bottom => gtk4_layer_shell::Layer::Bottom,
                Layer::Top => gtk4_layer_shell::Layer::Top,
                Layer::Overlay => gtk4_layer_shell::Layer::Overlay,
            }
        }
    }

    #[derive(clap::ValueEnum, serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub enum Anchor {
        Top,
        Bottom,
        Left,
        Right,
    }

    impl Into<gtk4_layer_shell::Edge> for &Anchor {
        fn into(self) -> gtk4_layer_shell::Edge {
            match self {
                Anchor::Top => gtk4_layer_shell::Edge::Top,
                Anchor::Bottom => gtk4_layer_shell::Edge::Bottom,
                Anchor::Left => gtk4_layer_shell::Edge::Left,
                Anchor::Right => gtk4_layer_shell::Edge::Right,
            }
        }
    }

    #[derive(Parser, serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct Config {
        pub namespace: Namespace,

        #[clap(long)]
        pub layer: Layer,

        #[clap(long)]
        pub anchors: Vec<Anchor>,

        #[clap(long)]
        pub auto_exclusive_zone: bool,

        #[serde(default)]
        pub monitor: Option<u32>,
    }
}

pub mod window {
    use crate::monitor::Monitor;

    use super::layer_shell;

    pub type Id = String;

    /// Template configuration for a window
    #[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
    pub struct Config<ConfigWrapper>
    where
        ConfigWrapper: std::fmt::Debug + Clone,
    {
        pub config: ConfigWrapper,
        pub layer_shell: Option<layer_shell::Config>,
        #[serde(default)]
        pub lazy: bool,
        #[serde(default)]
        pub monitor: Monitor,
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config<ConfigWrapper>
where
    ConfigWrapper: std::fmt::Debug + Clone,
{
    pub windows: HashMap<window::Id, window::Config<ConfigWrapper>>,
    // TODO: Add different layouts (which windows are active)
}
