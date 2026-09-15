//! Build audits for source files, styles, generated pages, and visitor privacy.

use anyhow::{Result, bail};
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn run(root: &Path, output: &Path) -> Result<()> {
    audit_source_files(root)?;
    audit_browser_privacy(root)?;
    audit_styles(root)?;
    audit_html(output)?;
    Ok(())
}

fn audit_browser_privacy(root: &Path) -> Result<()> {
    let source = fs::read_to_string(root.join("crates/webapp/src/lib.rs"))?;
    for forbidden in [
        "local_storage",
        "session_storage",
        "indexed_db",
        "send_beacon",
        "fetch_with",
        "WebSocket",
        "XmlHttpRequest",
        "user_agent",
        "geolocation",
        "hardware_concurrency",
        "device_memory",
        ".languages()",
        ".platform()",
        ".read_text(",
        ".read()",
    ] {
        if source.contains(forbidden) {
            bail!("browser data storage or network API `{forbidden}` is not permitted");
        }
    }
    Ok(())
}

fn audit_source_files(root: &Path) -> Result<()> {
    let mut files = Vec::new();
    collect_files(root, &mut files, &[".git", "build", "public", "target"])?;
    let forbidden: Vec<_> = files
        .into_iter()
        .filter(|path| {
            matches!(
                path.extension().and_then(OsStr::to_str),
                Some("js" | "jsx" | "ts" | "tsx")
            )
        })
        .collect();
    if !forbidden.is_empty() {
        bail!("handwritten JavaScript or TypeScript is forbidden: {forbidden:?}");
    }
    Ok(())
}

fn audit_styles(root: &Path) -> Result<()> {
    for name in sitegen::STYLE_MODULES {
        let path = root.join("styles").join(name);
        let css = fs::read_to_string(&path)?;
        if css.matches('{').count() != css.matches('}').count() {
            bail!("unbalanced CSS blocks in {}", path.display());
        }
        for declaration in css.split([';', '{', '}']) {
            if let Some((property, value)) = declaration.split_once(':')
                && property.trim().starts_with("text-decoration")
                && value.contains("underline")
            {
                bail!("link underlines are forbidden in {}", path.display());
            }
        }
        for remote in [
            "@import",
            "url(http",
            "url(\"http",
            "url('http",
            "url(//",
            "url(\"//",
            "url('//",
        ] {
            if css.contains(remote) {
                bail!(
                    "remote stylesheet resources are forbidden in {}",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

fn audit_html(public: &Path) -> Result<()> {
    let mut files = Vec::new();
    collect_files(public, &mut files, &[])?;
    for path in files
        .into_iter()
        .filter(|path| path.extension() == Some(OsStr::new("html")))
    {
        let html = fs::read_to_string(&path)?;
        if !html.to_ascii_lowercase().starts_with("<!doctype html>")
            || !html.contains("<main id=\"content\"")
        {
            bail!("malformed generated document {}", path.display());
        }
        if !html.contains("data-route=\"") {
            bail!("missing route metadata in {}", path.display());
        }
        for attribute in [
            "onclick=",
            "onchange=",
            "oninput=",
            "onkeydown=",
            "onkeyup=",
            "onload=",
        ] {
            if html.to_ascii_lowercase().contains(attribute) {
                bail!("inline event attribute in {}", path.display());
            }
        }
        let scripts: Vec<_> = html.match_indices("<script").collect();
        if scripts.len() != 1
            || !html.contains("<script type=\"module\" src=\"/assets/wasm/bootstrap.js?v=")
        {
            bail!("unexpected script entry point in {}", path.display());
        }
        audit_ids_and_fragments(&path, &html)?;
        audit_local_assets(public, &path, &html)?;
        if html.contains("<iframe") || html.contains("<embed") || html.contains("<object") {
            bail!(
                "embedded third-party content is forbidden in {}",
                path.display()
            );
        }
        for source in attribute_values(&html, "src=\"") {
            if !source.starts_with("/assets/") {
                bail!("subresources must be served locally in {}", path.display());
            }
        }
        for tag in html.split('<').filter(|tag| tag.starts_with("link ")) {
            if !tag.contains("rel=\"canonical\"") {
                let rel = attribute_values(tag, "rel=\"");
                let rel = rel.first().map(String::as_str).unwrap_or_default();
                for href in attribute_values(tag.split('>').next().unwrap_or_default(), "href=\"") {
                    if !allowed_link_resource(rel, &href) {
                        bail!("unapproved stylesheet or preconnect in {}", path.display());
                    }
                }
            }
        }
    }
    Ok(())
}

fn allowed_link_resource(rel: &str, href: &str) -> bool {
    href.starts_with("/assets/")
        || match rel {
            "stylesheet" => href.starts_with("https://fonts.googleapis.com/css2?"),
            "preconnect" => matches!(
                href,
                "https://fonts.googleapis.com" | "https://fonts.gstatic.com"
            ),
            _ => false,
        }
}

fn audit_ids_and_fragments(path: &Path, html: &str) -> Result<()> {
    let ids: BTreeSet<String> = attribute_values(html, "id=\"").into_iter().collect();
    let id_count = attribute_values(html, "id=\"").len();
    if ids.len() != id_count {
        bail!("duplicate HTML id in {}", path.display());
    }
    for fragment in attribute_values(html, "href=\"#") {
        if !fragment.is_empty() && !ids.contains(&fragment) {
            bail!("broken fragment #{fragment} in {}", path.display());
        }
    }
    Ok(())
}

fn audit_local_assets(public: &Path, path: &Path, html: &str) -> Result<()> {
    for marker in ["src=\"/assets/", "href=\"/assets/"] {
        for value in attribute_values(html, marker) {
            let relative = format!("assets/{}", value.split('?').next().unwrap_or_default());
            if !public.join(relative).is_file() {
                bail!("missing local asset referenced by {}", path.display());
            }
        }
    }
    Ok(())
}

fn attribute_values(input: &str, marker: &str) -> Vec<String> {
    input
        .split(marker)
        .skip(1)
        .filter_map(|rest| rest.split('"').next().map(str::to_owned))
        .collect()
}

fn collect_files(
    path: &Path,
    output: &mut Vec<PathBuf>,
    ignored_directories: &[&str],
) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir()
                && ignored_directories.contains(&entry.file_name().to_string_lossy().as_ref())
            {
                continue;
            }
            collect_files(&entry.path(), output, ignored_directories)?;
        }
    } else if path.is_file() {
        output.push(path.to_path_buf());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::allowed_link_resource;

    #[test]
    fn external_resource_exception_is_limited_to_google_fonts() {
        assert!(allowed_link_resource("stylesheet", "/assets/css/site.css"));
        assert!(allowed_link_resource(
            "stylesheet",
            "https://fonts.googleapis.com/css2?family=Albert+Sans&display=swap"
        ));
        for host in ["https://fonts.googleapis.com", "https://fonts.gstatic.com"] {
            assert!(allowed_link_resource("preconnect", host));
        }
        for (rel, href) in [
            (
                "stylesheet",
                "https://fonts.googleapis.com.example.test/css2?x",
            ),
            ("stylesheet", "https://example.test/style.css"),
            ("stylesheet", "http://fonts.googleapis.com/css2?x"),
            ("stylesheet", "https://fonts.googleapis.com/other?x"),
            ("preconnect", "https://fonts.gstatic.com.example.test"),
            ("preload", "https://fonts.googleapis.com/css2?x"),
        ] {
            assert!(!allowed_link_resource(rel, href), "accepted {rel}: {href}");
        }
    }
}
