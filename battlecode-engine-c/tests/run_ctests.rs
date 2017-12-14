use std::process::{Command};

// Run the makefile in ../ctests.
// This test always has `battlecode-engine-c` as its cwd.
#[test]
#[cfg(unix)]
fn run_c_tests() {
    let mut child = Command::new("make")
        .arg("-C")
        .arg("ctests")
        .spawn()
        .expect("failed to run make");
    let ecode = child.wait().expect("make failed");
    assert!(ecode.success());
}