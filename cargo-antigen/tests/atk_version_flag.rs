//! `cargo antigen --version` smoke test.
//!
//! Locks down the CLI version-flag contract added in rc.3: `--version` must
//! exit 0 and stdout must contain the workspace-pinned package version. This
//! is the gate that unblocks camp's version-mismatch warning sub-step, which
//! depends on being able to introspect the installed cargo-antigen version
//! from a subprocess invocation.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-antigen"))
}

#[test]
fn cargo_antigen_version_flag_exits_zero_and_prints_version() {
    let out = Command::new(bin())
        .arg("antigen")
        .arg("--version")
        .output()
        .expect("failed to run cargo-antigen");
    let exit = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert_eq!(exit, 0, "stdout={stdout} stderr={stderr}");
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "expected {} in stdout, got: {stdout}",
        env!("CARGO_PKG_VERSION"),
    );
}

#[test]
fn cargo_antigen_short_version_flag_works() {
    let out = Command::new(bin())
        .arg("antigen")
        .arg("-V")
        .output()
        .expect("failed to run cargo-antigen");
    let exit = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(exit, 0, "stdout={stdout}");
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "stdout={stdout}"
    );
}
