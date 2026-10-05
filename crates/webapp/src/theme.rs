//! Theme values and the only persistent browser preference.

/// The only preference persisted by the site.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Theme {
    Light,
    #[default]
    Dark,
}

impl Theme {
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    /// Ignores every cookie except an exact, valid theme preference.
    #[must_use]
    pub fn from_cookie(header: &str) -> Self {
        header
            .split(';')
            .find_map(|part| {
                let (name, value) = part.trim().split_once('=')?;
                (name == "theme").then(|| Self::parse(value)).flatten()
            })
            .unwrap_or_default()
    }

    /// Written only when the visitor explicitly switches themes.
    #[must_use]
    pub fn cookie(self, secure: bool) -> String {
        let secure_attribute = if secure { "; Secure" } else { "" };
        format!(
            "theme={}; Path=/; Max-Age=31536000; SameSite=Lax{secure_attribute}",
            self.as_str()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Theme;

    #[test]
    fn only_exact_valid_theme_cookie_values_are_used() {
        assert_eq!(
            Theme::from_cookie("other=light; theme=light; another=dark"),
            Theme::Light
        );
        assert_eq!(Theme::from_cookie("theme=dark"), Theme::Dark);
        for invalid in [
            "",
            "theme=",
            "theme=LIGHT",
            "theme=blue",
            "mytheme=light",
            "theme=light=extra",
            "theme",
        ] {
            assert_eq!(Theme::from_cookie(invalid), Theme::Dark);
        }
    }

    #[test]
    fn preference_cookie_contains_only_theme_and_fixed_attributes() {
        assert_eq!(
            Theme::Light.cookie(false),
            "theme=light; Path=/; Max-Age=31536000; SameSite=Lax"
        );
        assert_eq!(
            Theme::Dark.cookie(true),
            "theme=dark; Path=/; Max-Age=31536000; SameSite=Lax; Secure"
        );
    }
}
