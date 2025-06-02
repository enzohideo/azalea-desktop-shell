fn main() {
    relm4_icons_build::bundle_icons(
        "icons.rs",
        Some("br.usp.ime.Azalea"),
        None::<&str>,
        None::<&str>,
        ["play", "pause", "first", "last"],
    );
}
