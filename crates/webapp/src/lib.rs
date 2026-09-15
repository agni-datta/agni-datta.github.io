//! Theme preference, mobile menu, email contact, and citation copying for the static site.

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

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::Theme;
    use std::cell::RefCell;
    use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
    use wasm_bindgen_futures::{JsFuture, spawn_local};
    use web_sys::{Document, Element, Event, HtmlDocument, HtmlElement, KeyboardEvent};

    type EventHandler = Closure<dyn FnMut(Event)>;

    // Keep the three handlers alive for the lifetime of this document.
    thread_local! {
        static EVENT_HANDLERS: RefCell<Vec<EventHandler>> = const { RefCell::new(Vec::new()) };
    }

    #[derive(Clone)]
    struct App {
        document: Document,
        root: Element,
    }

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("window is unavailable")?;
        let document = window.document().ok_or("document is unavailable")?;
        let root = document
            .document_element()
            .ok_or("document root is unavailable")?;
        let app = App { document, root };
        app.restore_theme()?;

        let click_app = app.clone();
        let click: EventHandler = Closure::new(move |event: Event| {
            let _ = click_app.click(&event);
        });
        app.document
            .add_event_listener_with_callback("click", click.as_ref().unchecked_ref())?;

        let key_app = app.clone();
        let key: EventHandler = Closure::new(move |event: Event| {
            if let Some(event) = event.dyn_ref::<KeyboardEvent>() {
                let _ = key_app.keydown(event);
            }
        });
        app.document
            .add_event_listener_with_callback("keydown", key.as_ref().unchecked_ref())?;
        if let Some(button) = app.document.query_selector("[data-email-token]")? {
            button.remove_attribute("disabled")?;
        }

        // Back/forward can restore a cached document whose theme or menu is stale.
        let page: EventHandler = Closure::new(move |_: Event| {
            let _ = app.restore_theme();
            let _ = app.set_navigation(false, false);
        });
        window.add_event_listener_with_callback("pageshow", page.as_ref().unchecked_ref())?;
        EVENT_HANDLERS.with(|handlers| handlers.borrow_mut().extend([click, key, page]));
        Ok(())
    }

    impl App {
        fn restore_theme(&self) -> Result<(), JsValue> {
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

        fn navigation_open(&self) -> bool {
            self.root.get_attribute("data-navigation").as_deref() == Some("open")
        }

        fn set_navigation(&self, open: bool, move_focus: bool) -> Result<(), JsValue> {
            self.root
                .set_attribute("data-navigation", if open { "open" } else { "closed" })?;
            if let Some(button) = self.document.query_selector("[data-drawer-toggle]")? {
                button.set_attribute("aria-expanded", if open { "true" } else { "false" })?;
                button.set_attribute(
                    "aria-label",
                    if open {
                        "Close navigation"
                    } else {
                        "Open navigation"
                    },
                )?;
            }
            if move_focus {
                let selector = if open {
                    "#primary-nav a"
                } else {
                    "[data-drawer-toggle]"
                };
                if let Some(element) = self.document.query_selector(selector)? {
                    element.unchecked_ref::<HtmlElement>().focus()?;
                }
            }
            Ok(())
        }

        fn click(&self, event: &Event) -> Result<(), JsValue> {
            let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
            else {
                return Ok(());
            };
            if target.closest("[data-theme-toggle]")?.is_some() {
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
            } else if let Some(button) = target.closest("[data-copy-citation]")? {
                self.copy_citation(button)?;
            } else if let Some(button) = target.closest("[data-email-token]")? {
                let token = button.get_attribute("data-email-token").unwrap_or_default();
                let window = web_sys::window().ok_or("window is unavailable")?;
                // Reversible obfuscation: decode only for an explicit contact action.
                // Keep the address out of the page text and persistent DOM attributes.
                let address = window.atob(&token)?;
                window.open_with_url_and_target(&format!("mailto:{address}"), "_self")?;
            } else if target.closest("[data-drawer-toggle]")?.is_some() {
                self.set_navigation(!self.navigation_open(), true)?;
            } else if target.closest("[data-drawer-backdrop]")?.is_some() {
                self.set_navigation(false, true)?;
            } else if self.navigation_open() && target.closest("#primary-nav a")?.is_some() {
                self.set_navigation(false, false)?;
            }
            Ok(())
        }

        fn copy_citation(&self, button: Element) -> Result<(), JsValue> {
            let Some(citation) = button.closest(".paper-citation")? else {
                return Ok(());
            };
            let Some(entry) = citation.query_selector(".citation-entry code")? else {
                return Ok(());
            };
            let Some(status) = citation.query_selector(".citation-status")? else {
                return Ok(());
            };
            let fallback = "Select the entry to copy it manually.";
            let window = web_sys::window().ok_or("window is unavailable")?;
            let clipboard = window.navigator().clipboard();
            let value: &JsValue = clipboard.as_ref();
            if value.is_undefined() || value.is_null() {
                status.set_text_content(Some(fallback));
                return Ok(());
            }
            status.set_text_content(None);
            button.set_attribute("disabled", "")?;
            // Write only the selected citation, and only after an explicit click.
            // Call before spawning so browsers retain the user activation.
            let request = clipboard.write_text(&entry.text_content().unwrap_or_default());
            spawn_local(async move {
                let message = if JsFuture::from(request).await.is_ok() {
                    "BibTeX copied."
                } else {
                    fallback
                };
                status.set_text_content(Some(message));
                let _ = button.remove_attribute("disabled");
            });
            Ok(())
        }

        fn keydown(&self, event: &KeyboardEvent) -> Result<(), JsValue> {
            if !self.navigation_open() {
                return Ok(());
            }
            if event.key() == "Escape" {
                event.prevent_default();
                return self.set_navigation(false, true);
            }
            if event.key() == "Tab" {
                let controls = self
                    .document
                    .query_selector_all("[data-drawer-toggle], #primary-nav a")?;
                let count = controls.length();
                if count == 0 {
                    return Ok(());
                }
                if let Some(first) = controls.item(0)
                    && first
                        .unchecked_ref::<HtmlElement>()
                        .offset_parent()
                        .is_none()
                {
                    // Resizing to desktop makes the mobile controls inactive.
                    return self.set_navigation(false, false);
                }
                let active = self.document.active_element();
                let current = (0..count).find(|&index| {
                    controls
                        .item(index)
                        .is_some_and(|node| node.is_same_node(active.as_deref()))
                });
                let next = match current {
                    Some(index) if event.shift_key() => (index + count - 1) % count,
                    Some(index) => (index + 1) % count,
                    None => 0,
                };
                if let Some(destination) = controls.item(next) {
                    event.prevent_default();
                    destination.unchecked_ref::<HtmlElement>().focus()?;
                }
            }
            Ok(())
        }
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
