/// This file is a hack to prevent `cargo build` from failing if you don't run `make` first.
use std::path::Path;
use std::fs::File;

fn main() {
    println!("cargo:rerun-if-changed=");
    if !Path::new("src/bindings.rs").exists() {
        File::create("src/bindings.rs").unwrap();
    }
}
