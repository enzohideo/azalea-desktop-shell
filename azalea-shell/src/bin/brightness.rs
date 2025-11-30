use blight::{Change, Direction};

fn main() {
    blight::change_bl(5, Change::Regular, Direction::Inc, Some("ddcci6".into())).unwrap(); // Increases brightness by 5%
}
