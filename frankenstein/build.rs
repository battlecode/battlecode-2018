extern crate cc;
use std::process::Command;

fn run(cmd: &str, args: &[&str]) {
    let status = Command::new(cmd)
        .args(args)
        .status()
        .expect("failed to run");
    assert!(status.success());
}

/// Build script.
/// This does any extra stuff we need at build time.
fn main() {
    println!("rerun-if-changed=*.py");

    run("python3", &["generate.py"])
}