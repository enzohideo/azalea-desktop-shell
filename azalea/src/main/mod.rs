use azalea::{app, config::Config, model};

fn main() {
    app::run(Some(Config {
        windows: vec![model::Window {
            id: format!("default"),
            init: (),
        }],
    }));
}
