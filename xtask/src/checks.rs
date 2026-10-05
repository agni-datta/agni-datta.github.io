//! Formatting, Clippy, tests, and the production build.
use crate::{
    build,
    tools::{cargo, run},
};
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub(super) fn check(root: &Path) -> Result<()> {
    run(
        cargo(root)?
            .current_dir(root)
            .args(["fmt", "--all", "--", "--check"]),
        "checking Rust formatting",
    )?;
    run_dprint(root, true)?;
    run(
        cargo(root)?.current_dir(root).args([
            "clippy",
            "--locked",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ]),
        "checking native Rust targets",
    )?;
    run(
        cargo(root)?.current_dir(root).args([
            "clippy",
            "--locked",
            "--package",
            "webapp",
            "--target",
            "wasm32-unknown-unknown",
            "--",
            "-D",
            "warnings",
        ]),
        "checking browser Wasm",
    )?;
    run(
        cargo(root)?
            .current_dir(root)
            .args(["test", "--locked", "--workspace"]),
        "running Rust tests",
    )?;
    build::build(root)?;
    println!("all checks passed");
    Ok(())
}

pub(super) fn format(root: &Path) -> Result<()> {
    run(
        cargo(root)?.current_dir(root).args(["fmt", "--all"]),
        "formatting Rust",
    )?;
    run_dprint(root, false)
}

fn run_dprint(root: &Path, check_only: bool) -> Result<()> {
    let mut command = Command::new("dprint");
    command.current_dir(root);
    command.arg(if check_only { "check" } else { "fmt" });
    command.arg("--incremental=false");
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => bail!("dprint exited with {status}"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bail!("dprint is required; install the version pinned in .dprint-version")
        }
        Err(error) => Err(error).context("running dprint"),
    }
}
