pub mod layer_shell {
    use bincode::{Decode, Encode};
    use clap::Parser;

    pub type Namespace = String;

    #[derive(clap::ValueEnum, Encode, Decode, Debug, Clone)]
    pub enum Layer {
        Background,
        Bottom,
        Top,
        Overlay,
    }

    #[derive(clap::ValueEnum, Encode, Decode, Debug, Clone)]
    pub enum Anchor {
        Top,
        Bottom,
        Left,
        Right,
    }

    #[derive(Parser, Encode, Decode, Debug, Clone)]
    pub struct Model {
        #[clap(long)]
        layer: Option<Layer>,

        #[clap(long)]
        anchor: Option<Anchor>,
    }
}

pub mod window {
    use bincode::{Decode, Encode};
    use clap::Parser;

    use super::layer_shell;

    pub type Id = String;

    #[derive(Parser, Encode, Decode, Debug)]
    pub struct Init<InitData>
    where
        InitData:
            clap::Subcommand + bincode::enc::Encode + bincode::de::Decode<()> + std::fmt::Debug,
    {
        pub id: Id,

        #[command(subcommand)]
        pub init: InitData,

        #[command(flatten)]
        pub layer_shell: Option<layer_shell::Model>,
    }

    pub enum Action {
        Create { init: () },
    }
}
