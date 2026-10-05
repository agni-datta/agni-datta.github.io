//! Parse publication metadata while retaining exact BibTeX source for copying.
use anyhow::{ensure, Context, Result};
use biblatex::{Bibliography, RawBibliography};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct Parsed<'s> {
    pub(crate) entries: Bibliography,
    pub(crate) source_entries: BTreeMap<&'s str, &'s str>,
}

pub(crate) fn parse(source: &str) -> Result<Parsed<'_>> {
    let raw = RawBibliography::parse(source).context("parsing BibTeX")?;
    ensure!(
        raw.abbreviations.is_empty() && raw.preamble.is_empty(),
        "use self-contained entries in references.bib; @string and @preamble are unsupported"
    );
    let mut source_entries = BTreeMap::new();
    for entry in &raw.entries {
        let key = entry.v.key.v;
        ensure!(
            !key.is_empty()
                && !key
                    .chars()
                    .any(|ch| ch.is_whitespace() || "{}(),=\"".contains(ch)),
            "invalid citation key {key:?}"
        );
        // The parser's span stops immediately before the outer closing brace.
        let original = source
            .get(entry.span.start..=entry.span.end)
            .context("invalid BibTeX source span")?;
        ensure!(
            source_entries.insert(key, original).is_none(),
            "duplicate citation key {key}"
        );
        let mut fields = BTreeSet::new();
        for field in &entry.v.fields {
            let name = field.key.v.to_ascii_lowercase();
            ensure!(
                fields.insert(name.clone()),
                "{key}: duplicate BibTeX field {name}"
            );
            ensure!(
                name != "crossref" && name != "xdata",
                "{key}: use self-contained entries rather than {name}"
            );
        }
    }
    let entries = Bibliography::from_raw(raw).context("reading BibTeX fields")?;
    Ok(Parsed {
        entries,
        source_entries,
    })
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn selects_exact_entries_and_preserves_nested_fields() {
        let first = r#"@Misc{EPRINT:CamDat24,
  title = {{Fiat}-{Shamir} Goes Rational},
  note = "Quoted {nested \"text\"} and an @ sign",
  url = {https://example.test/a@b}
}"#;
        let second = r#"@Article {EPRINT:CamDat240,
  title = {A different paper with café and escaped \{braces\}}
}"#;
        let source = format!("% Source notes with @ignored{{\n{first}\n\n{second}\n");
        let result = parse(&source).unwrap().source_entries;
        assert_eq!(result.len(), 2);
        assert_eq!(result["EPRINT:CamDat24"], first);
        assert_eq!(result["EPRINT:CamDat240"], second);
        assert!(!result.contains_key("EPRINT:CamDat2"));
    }

    #[test]
    fn rejects_ambiguous_or_incomplete_entries() {
        for source in [
            "@Misc{same, title={First}}\n@Misc{same, title={Second}}",
            "@Misc{broken, title={Incomplete}",
            "@Misc{broken, title=\"Incomplete}",
            "@Misc{, title={Missing key}}",
            "@Misc{bad key, title={Invalid key}}",
            "@Misc(key, title={Parentheses})",
            "@String{venue={An external macro}}",
            "@Preamble{\"External setup\"}",
            "@Misc{duplicate, title={First}, TITLE={Second}}",
            "@Misc{child, crossref={parent}}",
            "@Misc{child, xdata={parent}}",
        ] {
            assert!(parse(source).is_err(), "accepted {source:?}");
        }
    }

    #[test]
    fn ignores_comments_and_accepts_an_empty_bibliography() {
        assert!(parse("% No publications yet\n")
            .unwrap()
            .source_entries
            .is_empty());
        let result = parse("@Comment{Source {notes}}\n@Misc{key, year={2024}}")
            .unwrap()
            .source_entries;
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("key"));
    }
}
