//! Typed section content and bibliography resolution.

use crate::publications::{self, Publications};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub(crate) struct SiteData {
    pub(crate) site: SiteMeta,
    profile: Profile,
    about: About,
    publications: Publications,
    teaching: EntryList<TeachingEntry>,
    service: EntryList<ServiceEntry>,
    talks: EntryList<TalkEntry>,
    notes: Notes,
    references: References,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SiteMeta {
    pub(crate) base_url: String,
    title: String,
    description: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    name: String,
    role: String,
    affiliation: String,
    affiliation_url: Option<String>,
    location: String,
    email_token: String,
    email_display: String,
    photo_path: Option<String>,
    photo_alt: Option<String>,
    links: Vec<Link>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Link {
    pub(crate) label: String,
    pub(crate) url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Author {
    pub(crate) name: String,
    pub(crate) url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct About {
    intro: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EntryList<T> {
    entries: Vec<T>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TeachingEntry {
    role: String,
    institution: String,
    institution_url: Option<String>,
    term: String,
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ServiceEntry {
    role: String,
    venues: Vec<String>,
    year: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TalkEntry {
    title: String,
    venue: String,
    venue_url: Option<String>,
    date: String,
    links: Vec<Link>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Notes {
    intro: String,
    entries: Vec<Note>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Note {
    title: String,
    description: String,
    authors: Vec<Author>,
    status: String,
    date: String,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct References {
    intro: String,
    groups: Vec<ReferenceGroup>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReferenceGroup {
    category: String,
    topics: Vec<ReferenceTopic>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReferenceTopic {
    title: String,
    items: Vec<Reference>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    title: String,
    url: String,
    note: String,
    institution: Option<String>,
    #[serde(default)]
    instructors: Vec<String>,
}

pub(crate) fn load(root: &Path) -> Result<SiteData> {
    let content = root.join("content");
    Ok(SiteData {
        site: load_toml(&content.join("site.toml"))?,
        profile: load_toml(&content.join("profile.toml"))?,
        about: load_toml(&content.join("about.toml"))?,
        publications: publications::load(root)?,
        teaching: load_toml(&content.join("teaching.toml"))?,
        service: load_toml(&content.join("service.toml"))?,
        talks: load_toml(&content.join("talks.toml"))?,
        notes: load_toml(&content.join("notes.toml"))?,
        references: load_toml(&content.join("references.toml"))?,
    })
}

pub(crate) fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let source = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&source).with_context(|| format!("parsing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{About, Notes, References};

    #[test]
    fn rejects_unrecognized_content_fields() {
        assert!(toml::from_str::<About>("intro = 'About'\ninterests_text = 'stale'").is_err());
        assert!(toml::from_str::<Notes>("intro = 'Notes'\nentries = []\nsections = []").is_err());
    }

    #[test]
    fn reads_nested_references_without_a_duplicate_view_model() {
        let source = "\
            intro = 'References'\n\
            [[groups]]\n\
            category = 'Books'\n\
            [[groups.topics]]\n\
            title = 'Complexity'\n\
            [[groups.topics.items]]\n\
            title = 'A book'\n\
            url = 'https://example.test/book'\n\
            note = 'A description'\n";
        let references: References = toml::from_str(source).unwrap();
        assert_eq!(references.groups[0].category, "Books");
        assert_eq!(references.groups[0].topics[0].title, "Complexity");
        let item = &references.groups[0].topics[0].items[0];
        assert_eq!(item.title, "A book");
        assert!(item.instructors.is_empty());
    }
}
