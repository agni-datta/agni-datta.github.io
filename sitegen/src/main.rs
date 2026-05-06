use anyhow::{Context as AnyhowContext, Result};
use chrono::Datelike;
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Result as TeraResult, Tera, Value};
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
struct SiteData {
    site: SiteMeta,
    person: Person,
    about: About,
    publications: Publications,
    teaching: Vec<TeachingItem>,
    service: Vec<ServiceItem>,
    talks: Vec<TalkItem>,
    resources: Resources,
}

#[derive(Debug, Deserialize, Serialize)]
struct SiteFile {
    site: SiteMeta,
    person: Person,
    about: About,
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
struct Resources {
    intro: String,
    sections: Vec<ResourceSection>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResourceSection {
    title: String,
    items: Vec<ResourceItem>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResourceItem {
    label: String,
    url: String,
    note: String,
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
    copy_if_exists(&root.join("CNAME"), &public_dir.join("CNAME"))?;
    copy_if_exists(&root.join("robots.txt"), &public_dir.join("robots.txt"))?;

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
    )?;
    render_page(
        &tera,
        &data,
        &build,
        "publications.html",
        &public_dir.join("publications/index.html"),
    )?;
    render_page(
        &tera,
        &data,
        &build,
        "resources.html",
        &public_dir.join("resources/index.html"),
    )?;
    render_page(
        &tera,
        &data,
        &build,
        "404.html",
        &public_dir.join("404.html"),
    )?;
    write_sitemap(&public_dir, &data.site.base_url)?;

    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .context("sitegen must be inside the workspace")
}

fn load_toml<T: for<'de> serde::Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}

fn load_site_data(root: &Path) -> Result<SiteData> {
    let content = root.join("content");
    let site: SiteFile = load_toml(&content.join("site.toml"))?;
    let publications = load_toml(&content.join("publications.toml"))?;
    let resources = load_toml(&content.join("resources.toml"))?;
    Ok(SiteData {
        site: site.site,
        person: site.person,
        about: site.about,
        publications,
        teaching: site.teaching,
        service: site.service,
        talks: site.talks,
        resources,
    })
}

fn render_page(
    tera: &Tera,
    data: &SiteData,
    build: &BuildInfo,
    template: &str,
    out_path: &Path,
) -> Result<()> {
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut ctx = Context::new();
    ctx.insert("data", data);
    ctx.insert("build", build);
    let rendered = tera
        .render(template, &ctx)
        .with_context(|| format!("rendering {template}"))?;
    fs::write(out_path, rendered).with_context(|| format!("writing {}", out_path.display()))
}

fn write_sitemap(public_dir: &Path, base_url: &str) -> Result<()> {
    let base = base_url.trim_end_matches('/');
    let mut body = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for page in ["/", "/publications/", "/resources/"] {
        body.push_str(&format!("  <url><loc>{base}{page}</loc></url>\n"));
    }
    body.push_str("</urlset>\n");
    fs::write(public_dir.join("sitemap.xml"), body).context("writing sitemap.xml")
}

fn copy_static(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(path.strip_prefix(src)?);
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

fn copy_if_exists(src: &Path, dst: &Path) -> Result<()> {
    if src.exists() {
        fs::copy(src, dst)
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
    let protected = protect_math(input);
    let parser = Parser::new_ext(&protected, options);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    restore_math(&out)
}

fn protect_math(input: &str) -> String {
    input
        .replace("\\(", "@@MJ_IO@@")
        .replace("\\)", "@@MJ_IC@@")
        .replace("\\[", "@@MJ_DO@@")
        .replace("\\]", "@@MJ_DC@@")
}

fn restore_math(input: &str) -> String {
    input
        .replace("@@MJ_IO@@", "\\(")
        .replace("@@MJ_IC@@", "\\)")
        .replace("@@MJ_DO@@", "\\[")
        .replace("@@MJ_DC@@", "\\]")
}

fn inline_markdown_to_html(input: &str) -> String {
    let rendered = markdown_to_html(input);
    let trimmed = rendered.trim();
    trimmed
        .strip_prefix("<p>")
        .and_then(|s| s.strip_suffix("</p>"))
        .unwrap_or(trimmed)
        .to_string()
}
