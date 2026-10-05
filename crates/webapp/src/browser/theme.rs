//! Restore and explicitly change the theme cookie.
use super::App;
use crate::Theme;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlDocument;

impl App {
    pub(super) fn restore_theme(&self) -> Result<(), JsValue> {
        let document: &HtmlDocument = self.document.unchecked_ref();
        self.apply_theme(Theme::from_cookie(&document.cookie().unwrap_or_default()))
    }

    fn apply_theme(&self, theme: Theme) -> Result<(), JsValue> {
        self.root.set_attribute("data-theme", theme.as_str())?;
        if let Some(button) = self.document.query_selector("[data-theme-toggle]")? {
            let label = format!("Switch to {} theme", theme.toggled().as_str());
            button.set_attribute("aria-label", &label)?;
            button.set_attribute("title", &label)?;
            button.set_attribute(
                "aria-pressed",
                if theme == Theme::Dark {
                    "true"
                } else {
                    "false"
                },
            )?;
        }
        Ok(())
    }

    pub(super) fn toggle_theme(&self) -> Result<(), JsValue> {
        let theme = self
            .root
            .get_attribute("data-theme")
            .and_then(|value| Theme::parse(&value))
            .unwrap_or_default()
            .toggled();
        self.apply_theme(theme)?;
        let document: &HtmlDocument = self.document.unchecked_ref();
        let secure = self
            .document
            .location()
            .and_then(|location| location.protocol().ok())
            .as_deref()
            == Some("https:");
        // A browser that denies cookies can still switch the current page.
        let _ = document.set_cookie(&theme.cookie(secure));
        Ok(())
    }
}
