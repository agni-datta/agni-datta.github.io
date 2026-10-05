//! Route rendering, Markdown filters, and sitemap generation.
use crate::{assets, content::SiteData};
use anyhow::{Context as AnyhowContext, Result};
use pulldown_cmark::{Options, Parser, html};
use serde::Serialize;
use std::fs;
use std::path::Path;
use tera::{Context, Kwargs, State, Tera};

// Route names match their templates; the error page is excluded from the sitemap.
pub(crate) const PAGES: &[(&str, &str)] = &[
    ("home", "/"),
    ("privacy", "/privacy/"),
    ("not-found", "/404.html"),
];

#[derive(Debug, Serialize)]
pub(crate) struct BuildInfo {
    pub(crate) year: i32,
    pub(crate) cache_key: String,
}

pub(crate) fn load_templates(root: &Path) -> Result<Tera> {
    let mut files = Vec::new();
    assets::collect_files(root, &mut files)?;
    files.sort();
    let mut sources = Vec::new();
    for path in files
        .into_iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "html"))
    {
        let name = path
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(&path)
            .with_context(|| format!("reading template {}", path.display()))?;
        sources.push((name, source));
    }
    let mut tera = Tera::new();
    tera.register_filter("markdown", markdown_filter);
    tera.register_filter("inline_markdown", inline_markdown_filter);
    tera.add_raw_templates(sources)?;
    Ok(tera)
}

pub(crate) fn render_page(
    tera: &Tera,
    data: &SiteData,
    build: &BuildInfo,
    current_route: &str,
    canonical_path: &str,
    out_path: &Path,
) -> Result<()> {
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut ctx = Context::new();
    ctx.insert("data", data);
    ctx.insert("build", build);
    let template = format!("pages/{current_route}.html");
    ctx.insert("canonical_path", canonical_path);
    ctx.insert("current_route", current_route);
    let rendered = tera
        .render(&template, &ctx)
        .with_context(|| format!("rendering {template}"))?;
    fs::write(out_path, rendered).with_context(|| format!("writing {}", out_path.display()))
}

pub(crate) fn write_sitemap(public_dir: &Path, base_url: &str) -> Result<()> {
    let base = base_url.trim_end_matches('/');
    let mut body = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for &(_, page) in PAGES.iter().filter(|(_, path)| path.ends_with('/')) {
        body.push_str(&format!("  <url><loc>{base}{page}</loc></url>\n"));
    }
    body.push_str("</urlset>\n");
    fs::write(public_dir.join("sitemap.xml"), body).context("writing sitemap.xml")
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
    use super::inline_markdown_to_html;

    #[test]
    fn inline_markdown_removes_only_the_wrapper_paragraph() {
        assert_eq!(
            inline_markdown_to_html("A [link](https://example.test)."),
            "A <a href=\"https://example.test\">link</a>."
        );
    }
}
