fn main() {
    // Woraround until we can use relative paths to set the docs images:
    // https://github.com/rust-lang/rust/issues/79254
    //
    // Solution from:
    // https://stackoverflow.com/a/67704588
    println!("cargo:rerun-if-changed=assets");
    let _ = std::fs::copy("../assets/azalea-logo.png", "../target/doc/azalea-logo.png");
}
