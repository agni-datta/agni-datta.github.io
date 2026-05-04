use anyhow::{Context as AnyhowContext, Result};
use chrono::Datelike;
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Result as TeraResult, Tera, Value};
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Serialize)]
struct SiteData {
    site: SiteMeta,
    person: Person,
    about: About,
    publications: Publications,
    teaching: Vec<TeachingItem>,
    service: Vec<ServiceItem>,
    talks: Vec<TalkItem>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SiteMeta {
    base_url: String,
    title: String,
    description: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Person {
    name: String,
    role: String,
    affiliation: String,
    affiliation_url: Option<String>,
    location: String,
    email: String,
    email_display: String,
    photo_path: Option<String>,
    photo_alt: Option<String>,
    github: String,
    google_scholar: Option<String>,
    links: Vec<Link>,
    advisors: Vec<PersonLink>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PersonLink {
    name: String,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Link {
    label: String,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct About {
    intro: String,
    interests_text: Option<String>,
    interests: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Publications {
    heading: String,
    manuscripts: Vec<Manuscript>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Manuscript {
    title: String,
    authors: Vec<PersonLink>,
    status: String,
    year: String,
    links: Vec<Link>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TeachingItem {
    role: String,
    institution: String,
    institution_url: Option<String>,
    term: String,
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ServiceItem {
    role: String,
    venue: String,
    venue_url: Option<String>,
    year: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TalkItem {
    title: String,
    venue: String,
    venue_url: Option<String>,
    date: String,
    links: Vec<Link>,
}

#[derive(Debug, Serialize)]
struct BuildInfo {
    year: i32,
}

fn main() -> Result<()> {
    let root = workspace_root()?;
    let data = load_site_data(&root)?;
    let public_dir = root.join("public");

    if public_dir.exists() {
        fs::remove_dir_all(&public_dir)
            .with_context(|| format!("removing {}", public_dir.display()))?;
    }
    fs::create_dir_all(&public_dir)
        .with_context(|| format!("creating {}", public_dir.display()))?;

    copy_static(&root.join("static"), &public_dir)?;
    fs::write(public_dir.join(".nojekyll"), "")?;
    copy_if_exists(root.join("CNAME"), public_dir.join("CNAME"))?;
    copy_if_exists(root.join("robots.txt"), public_dir.join("robots.txt"))?;

    let mut tera = Tera::new(root.join("templates/**/*.html").to_str().unwrap())?;
    tera.register_filter("markdown", markdown_filter);
    tera.register_filter("inline_markdown", inline_markdown_filter);

    let build = BuildInfo {
        year: chrono::Local::now().year(),
    };
    render_page(
        &tera,
        &data,
        &build,
        "index.html",
        &public_dir.join("index.html"),
        "home",
    )?;
    render_page(
        &tera,
        &data,
        &build,
        "publications.html",
        &public_dir.join("publications/index.html"),
        "publications",
    )?;
    render_page(
        &tera,
        &data,
        &build,
        "resources.html",
        &public_dir.join("resources/index.html"),
        "resources",
    )?;
    render_page(
        &tera,
        &data,
        &build,
        "404.html",
        &public_dir.join("404.html"),
        "",
    )?;
    write_sitemap(&public_dir, &data.site.base_url)?;

    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .context("sitegen must be built inside the workspace")
}

fn load_site_data(root: &Path) -> Result<SiteData> {
    let path = root.join("content/site.toml");
    let raw = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}

fn render_page(
    tera: &Tera,
    data: &SiteData,
    build: &BuildInfo,
    template: &str,
    out_path: &Path,
    current: &str,
) -> Result<()> {
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut ctx = Context::new();
    ctx.insert("data", data);
    ctx.insert("build", build);
    ctx.insert("current", current);
    let rendered = tera
        .render(template, &ctx)
        .with_context(|| format!("rendering {template}"))?;
    fs::write(out_path, rendered).with_context(|| format!("writing {}", out_path.display()))?;
    Ok(())
}

fn write_sitemap(public_dir: &Path, base_url: &str) -> Result<()> {
    let pages = ["/", "/publications/", "/resources/"];
    let mut body = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    for page in pages {
        body.push_str(&format!(
            "  <url><loc>{}{}</loc></url>\n",
            base_url.trim_end_matches('/'),
            page
        ));
    }
    body.push_str("</urlset>\n");
    fs::write(public_dir.join("sitemap.xml"), body)?;
    Ok(())
}

fn copy_static(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();
        let rel = path.strip_prefix(src)?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &target)
                .with_context(|| format!("copying {} to {}", path.display(), target.display()))?;
        }
    }
    Ok(())
}

fn copy_if_exists(src: PathBuf, dst: PathBuf) -> Result<()> {
    if src.exists() {
        fs::copy(&src, &dst)
            .with_context(|| format!("copying {} to {}", src.display(), dst.display()))?;
    }
    Ok(())
}

fn markdown_filter(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
    let Some(input) = value.as_str() else {
        return Ok(Value::String(String::new()));
    };
    Ok(Value::String(markdown_to_html(input)))
}

fn inline_markdown_filter(value: &Value, _: &HashMap<String, Value>) -> TeraResult<Value> {
    let Some(input) = value.as_str() else {
        return Ok(Value::String(String::new()));
    };
    Ok(Value::String(inline_markdown_to_html(input)))
}

fn markdown_to_html(input: &str) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_SMART_PUNCTUATION
        | Options::ENABLE_STRIKETHROUGH;

    let protected = protect_math_delimiters(input);
    let parser = Parser::new_ext(&protected, options);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    restore_math_delimiters(&out)
}

fn protect_math_delimiters(input: &str) -> String {
    input
        .replace("\\(", "@@MJ_INLINE_OPEN@@")
        .replace("\\)", "@@MJ_INLINE_CLOSE@@")
        .replace("\\[", "@@MJ_DISPLAY_OPEN@@")
        .replace("\\]", "@@MJ_DISPLAY_CLOSE@@")
}

fn restore_math_delimiters(input: &str) -> String {
    input
        .replace("@@MJ_INLINE_OPEN@@", "\\(")
        .replace("@@MJ_INLINE_CLOSE@@", "\\)")
        .replace("@@MJ_DISPLAY_OPEN@@", "\\[")
        .replace("@@MJ_DISPLAY_CLOSE@@", "\\]")
}

fn inline_markdown_to_html(input: &str) -> String {
    let rendered = markdown_to_html(input);
    let trimmed = rendered.trim();
    if let Some(inner) = trimmed
        .strip_prefix("<p>")
        .and_then(|s| s.strip_suffix("</p>"))
    {
        inner.to_string()
    } else {
        trimmed.to_string()
    }
}
