use azalea::{app, config::Config, model};

fn main() {
    app::run(Some(Config {
        windows: vec![model::Window {
            namespace: format!("default"),
            init: (),
        }],
    }));
}
