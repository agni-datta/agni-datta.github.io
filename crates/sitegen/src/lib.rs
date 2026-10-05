//! Static site generation from typed section content and reusable templates.
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

mod assets;
mod bibliography;
mod calendar;
mod content;
mod publications;
mod render;

pub use assets::STYLE_MODULES;
use assets::{build_cache_key, bundle_styles, copy_if_exists, copy_static};
use calendar::current_utc_year;
use render::{load_templates, render_page, write_sitemap, BuildInfo, PAGES};

/// Configuration for one deterministic site build.
#[derive(Clone, Debug)]
pub struct BuildConfig {
    /// Repository root containing `content`, `templates`, and `static`.
    pub workspace_root: PathBuf,
    /// Directory that receives the complete generated site.
    pub output_dir: PathBuf,
}

/// Summary of a completed site build.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildReport {
    /// Number of complete HTML documents emitted by the generator.
    pub page_count: usize,
    /// Fingerprint used by browser asset URLs.
    pub cache_key: String,
}

/// A site generation failure that leaves the previous output untouched.
#[derive(Debug, Error)]
pub enum BuildError {
    /// The build failed before an atomic output replacement was possible.
    #[error("site build failed: {0:#}")]
    Build(#[source] anyhow::Error),
}

/// Builds the complete static site and atomically replaces the configured output.
pub fn build(config: &BuildConfig) -> Result<BuildReport, BuildError> {
    build_inner(config).map_err(BuildError::Build)
}

fn build_inner(config: &BuildConfig) -> Result<BuildReport> {
    let root = &config.workspace_root;
    let data = content::load(root)?;
    let staging_parent = root.join("target/sitegen");
    let public_dir = staging_parent.join("public-next");

    if public_dir.exists() {
        fs::remove_dir_all(&public_dir)
            .with_context(|| format!("removing {}", public_dir.display()))?;
    }
    fs::create_dir_all(&public_dir)
        .with_context(|| format!("creating {}", public_dir.display()))?;

    copy_static(&root.join("static"), &public_dir)?;
    bundle_styles(root, &public_dir)?;
    copy_static(
        &root.join("target/site-assets/wasm"),
        &public_dir.join("assets/wasm"),
    )?;
    fs::write(public_dir.join(".nojekyll"), "")?;
    copy_if_exists(&root.join("CNAME"), &public_dir.join("CNAME"))?;
    copy_if_exists(&root.join("robots.txt"), &public_dir.join("robots.txt"))?;

    let tera = load_templates(&root.join("templates"))?;

    let build = BuildInfo {
        year: current_utc_year()?,
        cache_key: build_cache_key(root)?,
    };

    for &(name, path) in PAGES {
        let relative = path.trim_start_matches('/');
        let output = if path.ends_with('/') {
            public_dir.join(relative).join("index.html")
        } else {
            public_dir.join(relative)
        };
        render_page(&tera, &data, &build, name, path, &output)?;
    }
    write_sitemap(&public_dir, &data.site.base_url)?;
    replace_output(&public_dir, &config.output_dir)?;

    Ok(BuildReport {
        page_count: PAGES.len(),
        cache_key: build.cache_key,
    })
}

fn replace_output(staging: &Path, output: &Path) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating output parent {}", parent.display()))?;
    }
    let backup = output.with_extension("previous");
    if backup.exists() {
        fs::remove_dir_all(&backup)
            .with_context(|| format!("removing old backup {}", backup.display()))?;
    }
    if output.exists() {
        fs::rename(output, &backup)
            .with_context(|| format!("moving {} to {}", output.display(), backup.display()))?;
    }
    if let Err(error) = fs::rename(staging, output) {
        if backup.exists() {
            let _ = fs::rename(&backup, output);
        }
        return Err(error)
            .with_context(|| format!("moving staged build into {}", output.display()));
    }
    if backup.exists() {
        fs::remove_dir_all(&backup)
            .with_context(|| format!("removing backup {}", backup.display()))?;
    }
    Ok(())
}
