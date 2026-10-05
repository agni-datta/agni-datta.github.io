//! Publication cards derived from the shared BibTeX file.

use crate::{
    bibliography,
    content::{Author, Link, load_toml},
};
use anyhow::{Context, Result, ensure};
use biblatex::{ChunksExt, Entry};
use serde::{Deserialize, Serialize};
use std::{cmp::Reverse, collections::BTreeMap, fs, path::Path};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    heading: String,
    #[serde(default)]
    author_links: BTreeMap<String, String>,
    #[serde(default)]
    overrides: BTreeMap<String, DisplayOverride>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DisplayOverride {
    title: Option<String>,
    venue: Option<String>,
    year: Option<u16>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Publications {
    heading: String,
    entries: Vec<Publication>,
}

#[derive(Debug, Serialize)]
struct Publication {
    title: String,
    authors: Vec<Author>,
    status: String,
    year: String,
    links: Vec<Link>,
    citation: Citation,
}

#[derive(Debug, Serialize)]
struct Citation {
    key: String,
    version: String,
    entry: String,
}

pub(crate) fn load(root: &Path) -> Result<Publications> {
    let config = load_toml(&root.join("content/publications.toml"))?;
    let path = root.join("static/assets/bib/references.bib");
    let source =
        fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    from_bibtex(&source, &config)
        .with_context(|| format!("generating publications from {}", path.display()))
}

fn from_bibtex(source: &str, config: &Config) -> Result<Publications> {
    let bibliography = bibliography::parse(source)?;
    let mut records = Vec::new();
    for entry in bibliography.entries.iter() {
        let display = config.overrides.get(&entry.key);
        let bib_year: u16 = required_field(entry, "year")?
            .parse()
            .with_context(|| format!("{}: year must be a number", entry.key))?;
        let year = display.and_then(|value| value.year).unwrap_or(bib_year);
        ensure!(
            bib_year > 0 && year > 0,
            "{}: year must be positive",
            entry.key
        );
        let title = display
            .and_then(|value| value.title.clone())
            .unwrap_or(required_field(entry, "title")?);
        ensure!(
            !title.trim().is_empty(),
            "{}: title must not be empty",
            entry.key
        );
        let people = entry
            .author()
            .with_context(|| format!("{}: invalid or missing author", entry.key))?;
        ensure!(
            !people.is_empty(),
            "{}: author must not be empty",
            entry.key
        );
        ensure!(
            people.iter().all(|person| !person.name.is_empty()),
            "{}: author names must not be empty",
            entry.key
        );
        let authors = people
            .into_iter()
            .map(|person| {
                let name = person.to_string();
                Author {
                    url: config.author_links.get(&name).cloned(),
                    name,
                }
            })
            .collect();
        let links = paper_links(entry)?;
        let is_eprint = links.iter().any(|link| link.label == "ePrint");
        let status = display
            .and_then(|value| value.venue.clone())
            .unwrap_or_else(|| {
                ["booktitle", "journal", "journaltitle"]
                    .into_iter()
                    .find_map(|name| field(entry, name))
                    .unwrap_or_else(|| if is_eprint { "Preprint" } else { "Publication" }.into())
            });
        let version = format!(
            "{} · {bib_year}",
            if is_eprint {
                "ePrint version"
            } else {
                "Citation"
            }
        );
        let citation = Citation {
            key: entry.key.clone(),
            version,
            entry: bibliography.source_entries[entry.key.as_str()].to_owned(),
        };
        records.push((
            year,
            Publication {
                title,
                authors,
                status,
                year: year.to_string(),
                links,
                citation,
            },
        ));
    }
    // Stable sorting retains BibTeX order among papers displayed in the same year.
    records.sort_by_key(|(year, _)| Reverse(*year));
    Ok(Publications {
        heading: config.heading.clone(),
        entries: records
            .into_iter()
            .map(|(_, publication)| publication)
            .collect(),
    })
}

fn field(entry: &Entry, name: &str) -> Option<String> {
    entry
        .get(name)
        .map(ChunksExt::format_verbatim)
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn required_field(entry: &Entry, name: &str) -> Result<String> {
    field(entry, name).with_context(|| format!("{}: missing or empty {name}", entry.key))
}

fn paper_links(entry: &Entry) -> Result<Vec<Link>> {
    let mut links = Vec::new();
    if let Some(url) = field(entry, "url") {
        ensure!(
            url.starts_with("https://") || url.starts_with("http://"),
            "{}: url must use http or https",
            entry.key
        );
        let label = if url.starts_with("https://eprint.iacr.org/")
            || url.starts_with("http://eprint.iacr.org/")
        {
            "ePrint"
        } else if url
            .split(['?', '#'])
            .next()
            .is_some_and(|path| path.ends_with(".pdf"))
        {
            "PDF"
        } else {
            "Paper"
        };
        links.push(Link {
            label: label.into(),
            url,
        });
    }
    if let Some(doi) = field(entry, "doi") {
        let doi = doi
            .strip_prefix("https://doi.org/")
            .or_else(|| doi.strip_prefix("http://doi.org/"))
            .unwrap_or(&doi);
        ensure!(
            doi.starts_with("10.") && doi.contains('/'),
            "{}: invalid DOI",
            entry.key
        );
        let url = format!("https://doi.org/{doi}");
        if !links.iter().any(|link| link.url == url) {
            links.push(Link {
                label: "DOI".into(),
                url,
            });
        }
    }
    ensure!(
        !links.is_empty(),
        "{}: add a url or doi for the online paper",
        entry.key
    );
    Ok(links)
}

#[cfg(test)]
mod tests {
    use super::{Config, from_bibtex};

    fn config() -> Config {
        toml::from_str("heading = 'Publications'").unwrap()
    }

    const OLDER: &str = "@misc{older, title={{Fiat}-{Shamir} Goes Rational}, author={Campanelli, Matteo and Datta, Agni}, year=2024, url={https://eprint.iacr.org/2024/1645}}";
    const NEWER: &str = "@misc{newer, title={A new paper}, author={Agni Datta}, year={2026}, url={https://eprint.iacr.org/2026/2227}}";

    #[test]
    fn adding_editing_and_removing_bibtex_changes_the_list_without_toml_entries() {
        let config = config();
        let one = from_bibtex(OLDER, &config).unwrap();
        assert_eq!(one.entries.len(), 1);
        let two = from_bibtex(&format!("{OLDER}\n{NEWER}"), &config).unwrap();
        assert_eq!(two.entries.len(), 2);
        assert_eq!(two.entries[0].title, "A new paper");
        assert_eq!(two.entries[0].year, "2026");
        assert_eq!(
            two.entries[0].links[0].url,
            "https://eprint.iacr.org/2026/2227"
        );
        let edited =
            from_bibtex(&NEWER.replace("A new paper", "A revised title"), &config).unwrap();
        assert_eq!(edited.entries[0].title, "A revised title");
        assert!(
            from_bibtex("% No papers yet", &config)
                .unwrap()
                .entries
                .is_empty()
        );
    }

    #[test]
    fn reads_tex_names_and_preserves_the_original_citation() {
        let source = r#"@article{names, title={G{\"o}del and {NP}}, author={B{\"o}hm, Clara and Datta, Agni}, year="2025", journal={A journal}, doi={10.1000/example}}"#;
        let config: Config =
            toml::from_str("heading = 'Publications'\n[author_links]\n'Agni Datta' = '/'").unwrap();
        let result = from_bibtex(source, &config).unwrap();
        let paper = &result.entries[0];
        assert_eq!(paper.title, "Gödel and NP");
        assert_eq!(paper.authors[0].name, "Clara Böhm");
        assert_eq!(paper.authors[0].url, None);
        assert_eq!(paper.authors[1].url.as_deref(), Some("/"));
        assert_eq!(paper.status, "A journal");
        assert_eq!(paper.links[0].label, "DOI");
        assert_eq!(paper.links[0].url, "https://doi.org/10.1000/example");
        assert_eq!(paper.citation.entry, source);
    }

    #[test]
    fn display_overrides_do_not_change_preprint_citations_or_same_year_order() {
        let config: Config = toml::from_str("heading = 'Publications'\n[overrides.older]\ntitle = 'Full conference title'\nvenue = 'ASIACRYPT'\nyear = 2026").unwrap();
        let result = from_bibtex(&format!("{NEWER}\n{OLDER}"), &config).unwrap();
        let paper = &result.entries[1];
        assert_eq!(paper.title, "Full conference title");
        assert_eq!(paper.status, "ASIACRYPT");
        assert_eq!(paper.year, "2026");
        assert_eq!(paper.citation.version, "ePrint version · 2024");
        assert_eq!(paper.citation.entry, OLDER);
        // An old override cannot keep a removed paper on the site.
        assert_eq!(from_bibtex(NEWER, &config).unwrap().entries.len(), 1);
    }

    #[test]
    fn bibtex_additions_update_rendered_cards_and_download_count() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let tera = crate::render::load_templates(&root.join("templates")).unwrap();
        let source = format!("{OLDER}\n{NEWER}");
        let publications = from_bibtex(&source, &config()).unwrap();
        let mut context = tera::Context::new();
        context.insert(
            "data",
            &std::collections::BTreeMap::from([("publications", publications)]),
        );
        let html = tera.render("sections/publications.html", &context).unwrap();
        assert!(html.contains("Download all 2 papers"));
        assert_eq!(html.matches("class=\"entry paper\"").count(), 2);
        assert!(
            html.find("<h3>A new paper</h3>").unwrap()
                < html.find("<h3>Fiat-Shamir Goes Rational</h3>").unwrap()
        );
        assert!(!html.contains("href=\"\""));
        assert!(html.contains("Agni Datta"));

        let publications = from_bibtex(
            &NEWER.replace("A new paper", "A <b>new</b> title"),
            &config(),
        )
        .unwrap();
        context.insert(
            "data",
            &std::collections::BTreeMap::from([("publications", publications)]),
        );
        let html = tera.render("sections/publications.html", &context).unwrap();
        assert!(html.contains("Download all 1 paper"));
        assert_eq!(html.matches("class=\"entry paper\"").count(), 1);
        assert!(html.contains("A &lt;b&gt;new&lt;/b&gt; title"));
    }

    #[test]
    fn missing_publication_fields_fail_with_the_citation_key() {
        for (field, source) in [
            ("title", NEWER.replace("title={A new paper}, ", "")),
            ("author", NEWER.replace("author={Agni Datta}, ", "")),
            ("year", NEWER.replace("year={2026}, ", "")),
            (
                "url or doi",
                NEWER.replace(", url={https://eprint.iacr.org/2026/2227}", ""),
            ),
        ] {
            let error = from_bibtex(&source, &config()).unwrap_err().to_string();
            assert!(error.contains("newer"), "{error}");
            assert!(error.contains(field), "{error}");
        }
        assert!(toml::from_str::<Config>("heading = 'Publications'\nentries = []").is_err());
    }
}
