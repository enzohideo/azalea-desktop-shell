use azalea::{app, config::Config, model};

fn main() {
    app::run(Some(Config {
        windows: vec![model::window::Init {
            id: format!("default"),
            init: (),
            layer_shell: None,
        }],
    }));
}
