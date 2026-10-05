//! Case-insensitive publication queries, independent of the browser.

pub(crate) struct Query {
    terms: Vec<String>,
}

impl Query {
    pub(crate) fn new(input: &str) -> Self {
        Self {
            terms: normalize(input)
                .split_whitespace()
                .map(str::to_owned)
                .collect(),
        }
    }

    pub(crate) fn matches(&self, text: &str) -> bool {
        let text = normalize(text);
        self.terms.iter().all(|term| text.contains(term))
    }
}

fn normalize(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|character| match character {
            '‐' | '‑' | '‒' | '–' | '—' => '-',
            '‘' | '’' => '\'',
            _ => character,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::Query;

    const PAPER: &str =
        "Fiat–Shamir Goes Rational\nMatteo Campanelli, Agni Datta\nASIACRYPT · 2026";

    #[test]
    fn empty_queries_show_every_paper() {
        assert!(Query::new("").matches(PAPER));
        assert!(Query::new(" \t\n ").matches(PAPER));
    }

    #[test]
    fn terms_match_across_fields_regardless_of_case_and_order() {
        assert!(Query::new("2026 CAMPANELLI rational").matches(PAPER));
        assert!(Query::new("Asiacrypt datta").matches(PAPER));
        assert!(!Query::new("rational yogev").matches(PAPER));
        assert!(!Query::new("eurocrypt").matches(PAPER));
    }

    #[test]
    fn typographic_punctuation_matches_plain_keyboard_input() {
        assert!(Query::new("fiat-shamir").matches(PAPER));
        assert!(Query::new("researcher's").matches("Researcher’s notes"));
    }
}
