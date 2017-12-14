extern crate cheddar;

/// Build script.
/// This does any extra stuff we need at build time.
fn main() {
    println!("rerun-if-changed=src");
    println!("rerun-if-changed=src/*.rs");

    // Generate a c header file in `include` that specifies our C API.
    cheddar::Cheddar::new().expect("could not read manifest")
        .run_build("include/battlecode.h");

}
