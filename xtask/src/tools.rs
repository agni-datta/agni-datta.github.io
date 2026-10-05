//! Command execution using the toolchain pinned by this repository.
use anyhow::{Context, Result, bail};
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus};

pub(super) fn run(command: &mut Command, description: &str) -> Result<()> {
    let status: ExitStatus = command.status().with_context(|| description.to_owned())?;
    if !status.success() {
        bail!("{description} failed with {status}");
    }
    Ok(())
}

pub(super) fn cargo(root: &Path) -> Result<Command> {
    let source = fs::read_to_string(root.join("rust-toolchain.toml"))?;
    let config: toml::Table = toml::from_str(&source)?;
    let channel = config
        .get("toolchain")
        .and_then(|value| value.get("channel"))
        .and_then(toml::Value::as_str)
        .context("rust-toolchain.toml is missing toolchain.channel")?;
    let mut command = Command::new("rustup");
    command.args(["run", channel, "cargo"]);
    command.env("CARGO_TARGET_DIR", "target/rustup");
    if let Ok(output) = Command::new("rustup")
        .args(["which", "rustc", "--toolchain", channel])
        .output()
        && output.status.success()
    {
        let compiler = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        command.env("RUSTC", &compiler);
        if let Some(toolchain_bin) = Path::new(&compiler).parent() {
            let existing_path = env::var_os("PATH").unwrap_or_default();
            let paths = std::iter::once(toolchain_bin.to_path_buf())
                .chain(env::split_paths(&existing_path));
            if let Ok(path) = env::join_paths(paths) {
                command.env("PATH", path);
            }
        }
    }
    Ok(command)
}
