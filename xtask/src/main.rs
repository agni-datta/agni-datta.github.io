//! Cargo entry point for building, checking, formatting, and serving the site.
use anyhow::{Context, Result, bail};
use std::env;
use std::path::{Path, PathBuf};

mod audit;
mod build;
mod checks;
mod serve;
mod tools;

fn main() -> Result<()> {
    let root = workspace_root()?;
    let mut arguments = env::args().skip(1);
    let Some(command) = arguments.next() else {
        print_help();
        return Ok(());
    };
    let remaining: Vec<String> = arguments.collect();
    match command.as_str() {
        "build" => build::build(&root),
        "serve" => serve::serve(&root, &remaining),
        "check" => checks::check(&root),
        "format" => checks::format(&root),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        other => bail!("unknown site command `{other}`; run `cargo site help`"),
    }
}

fn print_help() {
    println!(
        "cargo site <command>\n\n  build      build browser Wasm and the static site atomically\n  serve      watch sources and serve http://localhost:8000\n  check      run formatting, Rust checks, tests, build, and source audits\n  format     format Rust, HTML, CSS, TOML, Markdown, and YAML"
    );
}

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .context("xtask must be a direct child of the workspace")
}
