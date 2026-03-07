use std::process::Command;

#[test]
fn version_prints_sage() {
    let output = Command::new(env!("CARGO_BIN_EXE_sage"))
        .arg("--version")
        .output()
        .expect("failed to run sage");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("sage"));
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn build_stub_prints_not_implemented() {
    let output = Command::new(env!("CARGO_BIN_EXE_sage"))
        .args(["build", "examples/hello.sg"])
        .output()
        .expect("failed to run sage");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("not implemented yet"));
}

#[test]
fn repl_stub_prints_not_implemented() {
    let output = Command::new(env!("CARGO_BIN_EXE_sage"))
        .arg("repl")
        .output()
        .expect("failed to run sage");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("not implemented yet"));
}
