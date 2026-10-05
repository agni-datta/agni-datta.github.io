//! Static assets, stylesheet bundling, and cache fingerprints.
use anyhow::{Context, Result};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

/// Ordered CSS sources, shared by bundling, asset fingerprints, and audits.
pub const STYLE_MODULES: &[&str] = &[
    "tokens.css",
    "foundation.css",
    "layout.css",
    "components/chrome.css",
    "components/controls.css",
    "components/sections.css",
    "components/entries.css",
    "components/disclosures.css",
    "sections/profile.css",
    "sections/about.css",
    "sections/publications.css",
    "sections/teaching.css",
    "sections/service.css",
    "sections/talks.css",
    "sections/notes.css",
    "sections/references.css",
    "responsive.css",
];

pub(crate) fn bundle_styles(root: &Path, public_dir: &Path) -> Result<()> {
    let styles = root.join("styles");
    let mut bundle = String::from("/* Generated from the ordered modules in styles/. */\n");
    for name in STYLE_MODULES {
        let path = styles.join(name);
        bundle.push_str(
            &fs::read_to_string(&path)
                .with_context(|| format!("reading style module {}", path.display()))?,
        );
        bundle.push('\n');
    }
    let output = public_dir.join("assets/css/site.css");
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, bundle).with_context(|| format!("writing {}", output.display()))
}

pub(crate) fn build_cache_key(root: &Path) -> Result<String> {
    let mut hasher = DefaultHasher::new();
    for name in STYLE_MODULES {
        let stylesheet = root.join("styles").join(name);
        fs::read(&stylesheet)
            .with_context(|| format!("hashing {}", stylesheet.display()))?
            .hash(&mut hasher);
    }
    let wasm_assets = root.join("target/site-assets/wasm");
    if wasm_assets.exists() {
        let mut files = Vec::new();
        collect_files(&wasm_assets, &mut files)?;
        files.sort();
        for path in files {
            path.strip_prefix(&wasm_assets)?.hash(&mut hasher);
            fs::read(&path)
                .with_context(|| format!("hashing {}", path.display()))?
                .hash(&mut hasher);
        }
    }
    Ok(format!("{:016x}", hasher.finish()))
}

pub(crate) fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

pub(crate) fn copy_static(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_static(&path, &target)?;
        } else {
            fs::copy(path, &target).with_context(|| format!("copying to {}", target.display()))?;
        }
    }
    Ok(())
}

pub(crate) fn copy_if_exists(src: &Path, dst: &Path) -> Result<()> {
    if src.exists() {
        fs::copy(src, dst)
            .with_context(|| format!("copying {} to {}", src.display(), dst.display()))?;
    }
    Ok(())
}
