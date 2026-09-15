use anyhow::{Context, Result, ensure};
use std::collections::BTreeMap;

/// Index complete, brace-delimited entries without rewriting their BibTeX.
/// The website bibliography uses self-contained entries, not string macros.
pub(super) fn entries(source: &str) -> Result<BTreeMap<&str, &str>> {
    let mut entries = BTreeMap::new();
    let mut rest = source.trim();
    while !rest.is_empty() {
        if rest.starts_with('%') {
            rest = rest
                .split_once('\n')
                .map_or("", |(_, tail)| tail.trim_start());
            continue;
        }
        ensure!(
            rest.starts_with('@'),
            "expected a BibTeX entry or % comment"
        );
        let open = rest.find('{').context("expected a brace-delimited entry")?;
        let kind = rest[1..open].trim();
        ensure!(
            !kind.is_empty() && kind.bytes().all(|byte| byte.is_ascii_alphabetic()),
            "expected a brace-delimited BibTeX entry"
        );
        ensure!(
            !kind.eq_ignore_ascii_case("string") && !kind.eq_ignore_ascii_case("preamble"),
            "use self-contained entries in references.bib; @{kind} is unsupported"
        );
        let end = entry_end(rest, open)?;
        if !kind.eq_ignore_ascii_case("comment") {
            let (key, _) = rest[open + 1..end - 1]
                .split_once(',')
                .context("expected a citation key followed by a comma")?;
            let key = key.trim();
            ensure!(
                !key.is_empty()
                    && !key
                        .chars()
                        .any(|ch| ch.is_whitespace() || "{}(),=\"".contains(ch)),
                "invalid citation key {key:?}"
            );
            ensure!(
                entries.insert(key, &rest[..end]).is_none(),
                "duplicate citation key {key}"
            );
        }
        rest = rest[end..].trim_start();
    }
    Ok(entries)
}

fn entry_end(source: &str, open: usize) -> Result<usize> {
    let mut depth = 1;
    let mut quoted = false;
    let mut escaped = false;
    for (offset, byte) in source.bytes().enumerate().skip(open + 1) {
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' => escaped = true,
            b'"' if depth == 1 => quoted = !quoted,
            b'{' => depth += 1,
            b'}' if depth > 1 => depth -= 1,
            b'}' if !quoted => return Ok(offset + 1),
            _ => {}
        }
    }
    anyhow::bail!("unclosed BibTeX entry beginning {:?}", &source[..=open])
}

#[cfg(test)]
mod tests {
    use super::entries;

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
        let result = entries(&source).unwrap();
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
        ] {
            assert!(entries(source).is_err(), "accepted {source:?}");
        }
    }

    #[test]
    fn ignores_comments_and_accepts_an_empty_bibliography() {
        assert!(entries("% No publications yet\n").unwrap().is_empty());
        let result = entries("@Comment{Source {notes}}\n@Misc{key, year={2024}}").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("key"));
    }
}
