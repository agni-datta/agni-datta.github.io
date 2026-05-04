use anyhow::{Context as AnyhowContext, Result};
use pulldown_cmark::{Options, Parser, html};
use serde::{Deserialize, Serialize};
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tera::{Context, Kwargs, State, Tera};
use thiserror::Error;

/// Configuration for one deterministic site build.
#[derive(Clone, Debug)]
pub struct BuildConfig {
    /// Repository root containing `content`, `templates`, and `static`.
    pub workspace_root: PathBuf,
    /// Directory that receives the complete generated site.
    pub output_dir: PathBuf,
}

impl BuildConfig {
    /// Returns the standard configuration used by local development and CI.
    pub fn discover() -> Result<Self, BuildError> {
        let workspace_root = workspace_root().map_err(BuildError::Build)?;
        Ok(Self {
            output_dir: workspace_root.join("public"),
            workspace_root,
        })
    }
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

#[derive(Debug, Serialize)]
struct SiteData {
    site: SiteMeta,
    person: Person,
    about: About,
    publications: Publications,
    notes: Notes,
    teaching: Vec<TeachingItem>,
    service: Vec<ServiceItem>,
    talks: Vec<TalkItem>,
    resources: Resources,
    miscellany: Miscellany,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PersonLink {
    name: String,
    url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Link {
    label: String,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct About {
    intro: String,
    interests_text: Option<String>,
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
    venues: Vec<String>,
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
    #[serde(skip_deserializing)]
    year_groups: Vec<PublicationYear>,
    #[serde(skip_deserializing)]
    recent: Vec<Manuscript>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Manuscript {
    title: String,
    authors: Vec<PersonLink>,
    status: String,
    year: String,
    links: Vec<Link>,
}

#[derive(Debug, Serialize)]
struct PublicationYear {
    year: String,
    manuscripts: Vec<Manuscript>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Notes {
    sections: Vec<NoteSection>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NoteSection {
    title: String,
    description: String,
    items: Vec<NoteItem>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NoteItem {
    title: String,
    description: String,
    authors: Vec<PersonLink>,
    status: String,
    date: String,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Resources {
    intro: String,
    sections: Vec<ResourceSection>,
    #[serde(skip_deserializing)]
    groups: Vec<ResourceGroup>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ResourceSection {
    category: String,
    title: String,
    items: Vec<ResourceItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ResourceItem {
    title: String,
    url: String,
    note: String,
    institution: Option<String>,
    #[serde(default)]
    instructors: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ResourceGroup {
    category: String,
    subtopics: Vec<ResourceSection>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Miscellany {
    intro: String,
    items: Vec<MiscellanyItem>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MiscellanyItem {
    title: String,
    description: String,
    icon: String,
}

#[derive(Debug, Serialize)]
struct BuildInfo {
    year: i32,
    cache_key: String,
}

/// Builds the complete static site and atomically replaces the configured output.
pub fn build(config: &BuildConfig) -> Result<BuildReport, BuildError> {
    build_inner(config).map_err(BuildError::Build)
}

fn build_inner(config: &BuildConfig) -> Result<BuildReport> {
    let root = &config.workspace_root;
    let data = load_site_data(root)?;
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

    render_page(
        &tera,
        &data,
        &build,
        "pages/home.html",
        &public_dir.join("index.html"),
    )?;
    render_page(
        &tera,
        &data,
        &build,
        "pages/publications.html",
        &public_dir.join("publications/index.html"),
    )?;
    render_page(
        &tera,
        &data,
        &build,
        "pages/notes.html",
        &public_dir.join("notes/index.html"),
    )?;
    render_page(
        &tera,
        &data,
        &build,
        "pages/resources.html",
        &public_dir.join("resources/index.html"),
    )?;
    render_page(
        &tera,
        &data,
        &build,
        "pages/miscellany.html",
        &public_dir.join("miscellany/index.html"),
    )?;
    render_page(
        &tera,
        &data,
        &build,
        "pages/privacy.html",
        &public_dir.join("privacy/index.html"),
    )?;
    render_page(
        &tera,
        &data,
        &build,
        "pages/not-found.html",
        &public_dir.join("404.html"),
    )?;
    write_sitemap(&public_dir, &data.site.base_url)?;
    replace_output(&public_dir, &config.output_dir)?;

    Ok(BuildReport {
        page_count: 7,
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

fn bundle_styles(root: &Path, public_dir: &Path) -> Result<()> {
    let styles = root.join("styles");
    if !styles.exists() {
        return Ok(());
    }
    let mut bundle = String::from("/* Generated from the ordered modules in styles/. */\n");
    for name in [
        "tokens.css",
        "foundation.css",
        "layout.css",
        "components.css",
        "responsive.css",
    ] {
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

fn build_cache_key(root: &Path) -> Result<String> {
    let mut hasher = DefaultHasher::new();
    for name in [
        "tokens.css",
        "foundation.css",
        "layout.css",
        "components.css",
        "responsive.css",
    ] {
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

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
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

fn current_utc_year() -> Result<i32> {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock predates the Unix epoch")?
        .as_secs()
        / 86_400;
    Ok(year_from_days_since_epoch(days))
}

fn year_from_days_since_epoch(mut days: u64) -> i32 {
    let mut year = 1970;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            return year;
        }
        days -= days_in_year;
        year += 1;
    }
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("sitegen must be inside the workspace")
}

fn load_templates(root: &Path) -> Result<Tera> {
    let mut sources = Vec::new();
    collect_template_sources(root, root, &mut sources)?;
    sources.sort_by(|left, right| left.0.cmp(&right.0));

    let mut tera = Tera::new();
    tera.register_filter("markdown", markdown_filter);
    tera.register_filter("inline_markdown", inline_markdown_filter);
    tera.add_raw_templates(sources)?;
    Ok(tera)
}

fn collect_template_sources(
    root: &Path,
    directory: &Path,
    sources: &mut Vec<(String, String)>,
) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_template_sources(root, &path, sources)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("html") {
            let name = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            let source = fs::read_to_string(&path)
                .with_context(|| format!("reading template {}", path.display()))?;
            sources.push((name, source));
        }
    }
    Ok(())
}

fn load_toml<T: for<'de> serde::Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}

fn load_site_data(root: &Path) -> Result<SiteData> {
    let content = root.join("content");
    let site: SiteFile = load_toml(&content.join("site.toml"))?;
    let mut publications: Publications = load_toml(&content.join("publications.toml"))?;
    publications.recent = publications.manuscripts.iter().take(10).cloned().collect();
    for manuscript in &publications.manuscripts {
        if let Some(group) = publications
            .year_groups
            .iter_mut()
            .find(|group| group.year == manuscript.year)
        {
            group.manuscripts.push(manuscript.clone());
        } else {
            publications.year_groups.push(PublicationYear {
                year: manuscript.year.clone(),
                manuscripts: vec![manuscript.clone()],
            });
        }
    }
    let notes = load_toml(&content.join("notes.toml"))?;
    let mut resources: Resources = load_toml(&content.join("resources.toml"))?;
    for section in &resources.sections {
        if let Some(group) = resources
            .groups
            .iter_mut()
            .find(|group| group.category == section.category)
        {
            group.subtopics.push(section.clone());
        } else {
            resources.groups.push(ResourceGroup {
                category: section.category.clone(),
                subtopics: vec![section.clone()],
            });
        }
    }
    let miscellany = load_toml(&content.join("miscellany.toml"))?;
    Ok(SiteData {
        site: site.site,
        person: site.person,
        about: site.about,
        publications,
        notes,
        teaching: site.teaching,
        service: site.service,
        talks: site.talks,
        resources,
        miscellany,
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
    let current_route = match template {
        "pages/home.html" => "home",
        "pages/publications.html" => "publications",
        "pages/notes.html" => "notes",
        "pages/resources.html" => "resources",
        "pages/miscellany.html" => "miscellany",
        "pages/privacy.html" => "privacy",
        _ => "not-found",
    };
    ctx.insert("current_route", current_route);
    for route in ["home", "publications", "notes", "resources", "miscellany"] {
        ctx.insert(
            format!("current_{route}"),
            if current_route == route {
                "page"
            } else {
                "false"
            },
        );
    }
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
    for page in [
        "/",
        "/publications/",
        "/notes/",
        "/resources/",
        "/miscellany/",
        "/privacy/",
    ] {
        body.push_str(&format!("  <url><loc>{base}{page}</loc></url>\n"));
    }
    body.push_str("</urlset>\n");
    fs::write(public_dir.join("sitemap.xml"), body).context("writing sitemap.xml")
}

fn copy_static(src: &Path, dst: &Path) -> Result<()> {
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

fn copy_if_exists(src: &Path, dst: &Path) -> Result<()> {
    if src.exists() {
        fs::copy(src, dst)
            .with_context(|| format!("copying {} to {}", src.display(), dst.display()))?;
    }
    Ok(())
}

fn markdown_filter(value: &str, _: Kwargs, _: &State) -> String {
    markdown_to_html(value)
}

fn inline_markdown_filter(value: &str, _: Kwargs, _: &State) -> String {
    inline_markdown_to_html(value)
}

fn markdown_to_html(input: &str) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_SMART_PUNCTUATION
        | Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(input, options);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
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

#[cfg(test)]
mod tests {
    use super::{
        inline_markdown_to_html, is_leap_year, load_site_data, workspace_root,
        year_from_days_since_epoch,
    };

    #[test]
    fn computes_year_from_unix_days() {
        assert_eq!(year_from_days_since_epoch(0), 1970);
        assert_eq!(year_from_days_since_epoch(364), 1970);
        assert_eq!(year_from_days_since_epoch(365), 1971);
        assert_eq!(year_from_days_since_epoch(10_957), 2000);
        assert_eq!(year_from_days_since_epoch(11_323), 2001);
    }

    #[test]
    fn applies_gregorian_leap_year_rules() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2025));
    }

    #[test]
    fn inline_markdown_removes_only_the_wrapper_paragraph() {
        assert_eq!(
            inline_markdown_to_html("A [link](https://example.test)."),
            "A <a href=\"https://example.test\">link</a>."
        );
    }

    #[test]
    fn content_grouping_preserves_every_source_item() {
        let root = workspace_root().unwrap();
        let data = load_site_data(&root).unwrap();
        let grouped_publications: usize = data
            .publications
            .year_groups
            .iter()
            .map(|group| group.manuscripts.len())
            .sum();
        let grouped_resources: usize = data
            .resources
            .groups
            .iter()
            .flat_map(|group| &group.subtopics)
            .map(|section| section.items.len())
            .sum();
        let source_resources: usize = data
            .resources
            .sections
            .iter()
            .map(|section| section.items.len())
            .sum();
        assert_eq!(grouped_publications, data.publications.manuscripts.len());
        assert_eq!(grouped_resources, source_resources);
    }
}
