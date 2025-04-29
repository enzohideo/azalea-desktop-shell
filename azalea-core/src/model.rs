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

    #[derive(clap::ValueEnum, serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub enum Anchor {
        Top,
        Bottom,
        Left,
        Right,
    }

    #[derive(Parser, serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct Model {
        #[clap(long)]
        layer: Option<Layer>,

        #[clap(long)]
        anchor: Option<Anchor>,
    }
}

pub mod window {
    use clap::Parser;

    use super::layer_shell;

    pub type Id = String;

    #[derive(Parser, serde::Serialize, serde::Deserialize, Debug)]
    pub struct InitDTO<InitWrapper>
    where
        InitWrapper: clap::Subcommand + std::fmt::Debug,
    {
        pub id: Id,

        #[command(subcommand)]
        pub init: InitWrapper,

        #[command(flatten)]
        pub layer_shell: Option<layer_shell::Model>,
    }
}
