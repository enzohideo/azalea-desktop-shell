use azalea::{app, config::Config, model};

fn main() {
    app::run(Some(Config {
        windows: vec![model::Window {
            title: format!("default"),
            init: (),
        }],
    }));
}
