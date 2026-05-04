//! Browser state and interaction runtime for the generated static site.

use std::collections::BTreeMap;

/// The two color themes supported by the site.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Theme {
    /// The light color palette.
    Light,
    /// The dark color palette, which remains the default for new visitors.
    #[default]
    Dark,
}

impl Theme {
    /// Parses the stable value used by cookies and legacy browser storage.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }

    /// Returns the stable value used by the DOM and theme cookie.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    /// Returns the opposite theme.
    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

/// The known route associated with the current static document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Route {
    Home,
    Publications,
    Notes,
    Resources,
    Miscellany,
    Privacy,
    NotFound,
}

impl Route {
    /// Classifies a URL path without allowing the route set to grow from user input.
    #[must_use]
    pub fn from_path(path: &str) -> Self {
        match path.trim_end_matches('/') {
            "" => Self::Home,
            "/publications" => Self::Publications,
            "/notes" => Self::Notes,
            "/resources" => Self::Resources,
            "/miscellany" => Self::Miscellany,
            "/privacy" => Self::Privacy,
            _ => Self::NotFound,
        }
    }

    /// Returns the value used by navigation metadata in generated HTML.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Publications => "publications",
            Self::Notes => "notes",
            Self::Resources => "resources",
            Self::Miscellany => "miscellany",
            Self::Privacy => "privacy",
            Self::NotFound => "not-found",
        }
    }
}

/// Whether a same-origin page request is currently replacing the main content.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NavigationStatus {
    #[default]
    Idle,
    Loading,
}

/// The complete long-lived browser state. Page replacements never recreate this value.
#[derive(Debug)]
pub struct AppState {
    /// Effective theme applied to the document root.
    pub theme: Theme,
    /// Bounded route represented by the current main content.
    pub route: Route,
    /// Whether the mobile navigation drawer is open.
    pub navigation_open: bool,
    /// Current same-origin navigation lifecycle state.
    pub navigation_status: NavigationStatus,
    /// Monotonic identifier used to reject stale overlapping responses.
    pub request_generation: u64,
    scroll_positions: BTreeMap<String, f64>,
}

impl AppState {
    /// Creates state for a freshly loaded static page.
    #[must_use]
    pub fn new(theme: Theme, route: Route) -> Self {
        Self {
            theme,
            route,
            navigation_open: false,
            navigation_status: NavigationStatus::Idle,
            request_generation: 0,
            scroll_positions: BTreeMap::new(),
        }
    }

    /// Starts a request and returns the generation that owns its eventual response.
    pub fn begin_request(&mut self) -> u64 {
        self.request_generation = self.request_generation.wrapping_add(1);
        self.navigation_status = NavigationStatus::Loading;
        self.request_generation
    }

    /// Returns whether a response still belongs to the most recent request.
    #[must_use]
    pub const fn owns_request(&self, generation: u64) -> bool {
        self.request_generation == generation
    }

    /// Remembers the vertical scroll position for a complete URL.
    pub fn remember_scroll(&mut self, url: String, position: f64) {
        self.scroll_positions.insert(url, position.max(0.0));
    }

    /// Retrieves a remembered scroll position, defaulting to the top of the page.
    #[must_use]
    pub fn scroll_for(&self, url: &str) -> f64 {
        self.scroll_positions.get(url).copied().unwrap_or(0.0)
    }
}

/// The result of resolving the first-party cookie and one-time legacy value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThemeResolution {
    /// Theme selected by cookie, migration, or the dark default.
    pub theme: Theme,
    /// Whether a valid legacy value must be persisted and removed.
    pub migrated_legacy_value: bool,
}

/// Centralized parser and serializer policy for theme persistence.
pub struct ThemeStore;

impl ThemeStore {
    /// Resolves the theme with cookie precedence, then legacy storage, then dark default.
    #[must_use]
    pub fn resolve(cookie_header: &str, legacy_value: Option<&str>) -> ThemeResolution {
        if let Some(theme) = parse_theme_cookie(cookie_header) {
            return ThemeResolution {
                theme,
                migrated_legacy_value: false,
            };
        }
        if let Some(theme) = legacy_value.and_then(Theme::parse) {
            return ThemeResolution {
                theme,
                migrated_legacy_value: true,
            };
        }
        ThemeResolution {
            theme: Theme::Dark,
            migrated_legacy_value: false,
        }
    }

    /// Builds the exact first-party cookie written after an explicit change or migration.
    #[must_use]
    pub fn cookie(theme: Theme, secure: bool) -> String {
        let secure_attribute = if secure { "; Secure" } else { "" };
        format!(
            "theme={}; Path=/; Max-Age=31536000; SameSite=Lax{secure_attribute}",
            theme.as_str()
        )
    }
}

fn parse_theme_cookie(header: &str) -> Option<Theme> {
    header.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        (name == "theme").then(|| Theme::parse(value)).flatten()
    })
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::{AppState, NavigationStatus, Route, Theme, ThemeStore};
    use js_sys::JsString;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::{JsFuture, spawn_local};
    use web_sys::{
        Document, DomParser, Element, Event, HtmlAnchorElement, HtmlDocument, HtmlElement,
        KeyboardEvent, MouseEvent, Response, ScrollBehavior, ScrollRestoration, ScrollToOptions,
        SupportedType, Url, Window,
    };

    type EventHandler = Closure<dyn FnMut(Event)>;

    thread_local! {
        static EVENT_HANDLERS: RefCell<Vec<EventHandler>> = RefCell::new(Vec::new());
    }

    #[derive(Clone)]
    struct App {
        window: Window,
        document: Document,
        state: Rc<RefCell<AppState>>,
    }

    #[derive(Clone, Copy)]
    enum HistoryAction {
        Push,
        Preserve,
    }

    /// Initializes theme and interaction state before revealing the document.
    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("document is unavailable"))?;
        let html_document: HtmlDocument = document.clone().unchecked_into();
        let cookie = html_document.cookie().unwrap_or_default();
        let legacy = window
            .local_storage()?
            .and_then(|storage| storage.get_item("theme").ok().flatten());
        let resolution = ThemeStore::resolve(&cookie, legacy.as_deref());

        if legacy.as_deref().and_then(Theme::parse).is_some()
            && let Some(storage) = window.local_storage()?
        {
            storage.remove_item("theme")?;
        }
        if resolution.migrated_legacy_value {
            persist_theme(&window, &document, resolution.theme)?;
        }

        let route = Route::from_path(&window.location().pathname()?);
        let app = App {
            window,
            document,
            state: Rc::new(RefCell::new(AppState::new(resolution.theme, route))),
        };
        if let Ok(history) = app.window.history() {
            history.set_scroll_restoration(ScrollRestoration::Manual)?;
        }
        app.apply_theme()?;
        app.set_navigation(false)?;
        app.mark_active_navigation()?;
        app.install_event_handlers()?;
        Ok(())
    }

    impl App {
        fn install_event_handlers(&self) -> Result<(), JsValue> {
            let click_app = self.clone();
            let click = Closure::wrap(Box::new(move |event: Event| {
                let Some(mouse) = event.dyn_ref::<MouseEvent>() else {
                    return;
                };
                if let Err(error) = click_app.handle_click(mouse) {
                    web_sys::console::error_1(&error);
                }
            }) as Box<dyn FnMut(Event)>);
            self.document
                .add_event_listener_with_callback("click", click.as_ref().unchecked_ref())?;

            let key_app = self.clone();
            let keydown = Closure::wrap(Box::new(move |event: Event| {
                let Some(keyboard) = event.dyn_ref::<KeyboardEvent>() else {
                    return;
                };
                if keyboard.key() == "Escape" {
                    let _ = key_app.set_navigation(false);
                }
            }) as Box<dyn FnMut(Event)>);
            self.document
                .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())?;

            let pop_app = self.clone();
            let popstate = Closure::wrap(Box::new(move |_event: Event| {
                let url = pop_app.window.location().href().unwrap_or_default();
                let app = pop_app.clone();
                spawn_local(async move {
                    app.navigate(url, HistoryAction::Preserve, true).await;
                });
            }) as Box<dyn FnMut(Event)>);
            self.window
                .add_event_listener_with_callback("popstate", popstate.as_ref().unchecked_ref())?;

            EVENT_HANDLERS
                .with(|handlers| handlers.borrow_mut().extend([click, keydown, popstate]));
            Ok(())
        }

        fn handle_click(&self, event: &MouseEvent) -> Result<(), JsValue> {
            let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<Element>().ok())
            else {
                return Ok(());
            };

            if target.closest("[data-theme-toggle]")?.is_some() {
                event.prevent_default();
                let next = self.state.borrow().theme.toggled();
                self.state.borrow_mut().theme = next;
                persist_theme(&self.window, &self.document, next)?;
                self.apply_theme()?;
                return Ok(());
            }
            if target.closest("[data-drawer-toggle]")?.is_some() {
                event.prevent_default();
                let next = !self.state.borrow().navigation_open;
                self.set_navigation(next)?;
                return Ok(());
            }
            if target.closest("[data-drawer-backdrop]")?.is_some() {
                event.prevent_default();
                self.set_navigation(false)?;
                return Ok(());
            }

            let Some(anchor) = target
                .closest("a[href]")?
                .and_then(|element| element.dyn_into::<HtmlAnchorElement>().ok())
            else {
                return Ok(());
            };
            if !is_unmodified_primary_click(event)
                || anchor.has_attribute("download")
                || !matches!(anchor.target().as_str(), "" | "_self")
            {
                return Ok(());
            }

            let base = self
                .document
                .base_uri()?
                .unwrap_or(self.window.location().href()?);
            let destination = Url::new_with_base(&anchor.href(), &base)?;
            let current = Url::new(&self.window.location().href()?)?;
            if destination.origin() != current.origin()
                || !matches!(destination.protocol().as_str(), "http:" | "https:")
                || destination
                    .pathname()
                    .to_ascii_lowercase()
                    .ends_with(".pdf")
            {
                return Ok(());
            }

            if destination.pathname() == current.pathname()
                && destination.search() == current.search()
                && !destination.hash().is_empty()
            {
                if self.scroll_to_fragment(&destination.hash()) {
                    event.prevent_default();
                    self.window.history()?.push_state_with_url(
                        &JsValue::NULL,
                        "",
                        Some(&destination.href()),
                    )?;
                }
                return Ok(());
            }

            event.prevent_default();
            self.set_navigation(false)?;
            let app = self.clone();
            spawn_local(async move {
                app.navigate(destination.href(), HistoryAction::Push, false)
                    .await;
            });
            Ok(())
        }

        async fn navigate(&self, url: String, history: HistoryAction, restore_scroll: bool) {
            let previous_url = self.window.location().href().unwrap_or_default();
            let previous_scroll = self.window.scroll_y().unwrap_or(0.0);
            let generation = {
                let mut state = self.state.borrow_mut();
                state.remember_scroll(previous_url, previous_scroll);
                state.begin_request()
            };
            let _ = self.set_navigation_status(NavigationStatus::Loading);

            let result = self.fetch_document(&url).await;
            if !self.state.borrow().owns_request(generation) {
                return;
            }
            match result {
                Ok(incoming) => {
                    if self.commit_document(&incoming, &url, history).is_err() {
                        let _ = self.window.location().assign(&url);
                        return;
                    }
                    self.state.borrow_mut().navigation_status = NavigationStatus::Idle;
                    let _ = self.set_navigation_status(NavigationStatus::Idle);
                    if let Ok(parsed) = Url::new(&url)
                        && !parsed.hash().is_empty()
                        && self.scroll_to_fragment(&parsed.hash())
                    {
                        return;
                    }
                    let top = if restore_scroll {
                        self.state.borrow().scroll_for(&url)
                    } else {
                        0.0
                    };
                    self.window.scroll_to_with_x_and_y(0.0, top);
                }
                Err(_) => {
                    let _ = self.window.location().assign(&url);
                }
            }
        }

        async fn fetch_document(&self, url: &str) -> Result<Document, JsValue> {
            let value = JsFuture::from(self.window.fetch_with_str(url)).await?;
            let response: Response = value.dyn_into()?;
            let content_type = response.headers().get("content-type")?.unwrap_or_default();
            if !response.ok() || !content_type.contains("text/html") {
                return Err(JsValue::from_str(
                    "navigation response was not successful HTML",
                ));
            }
            let text = JsFuture::from(response.text()?).await?;
            let text = text
                .dyn_into::<JsString>()?
                .as_string()
                .ok_or_else(|| JsValue::from_str("response text was unavailable"))?;
            DomParser::new()?.parse_from_string(&text, SupportedType::TextHtml)
        }

        fn commit_document(
            &self,
            incoming: &Document,
            url: &str,
            history: HistoryAction,
        ) -> Result<(), JsValue> {
            let current_main = self
                .document
                .query_selector("main#content")?
                .ok_or_else(|| JsValue::from_str("current main content is missing"))?;
            let incoming_main = incoming
                .query_selector("main#content")?
                .ok_or_else(|| JsValue::from_str("incoming main content is missing"))?;
            current_main.set_inner_html(&incoming_main.inner_html());
            if let Some(route) = incoming_main.get_attribute("data-route") {
                current_main.set_attribute("data-route", &route)?;
            }
            self.document.set_title(&incoming.title());
            if let (Some(current), Some(next)) = (
                self.document.query_selector("link[rel='canonical']")?,
                incoming.query_selector("link[rel='canonical']")?,
            ) && let Some(href) = next.get_attribute("href")
            {
                current.set_attribute("href", &href)?;
            }
            if matches!(history, HistoryAction::Push) {
                self.window
                    .history()?
                    .push_state_with_url(&JsValue::NULL, "", Some(url))?;
            }
            self.state.borrow_mut().route = Route::from_path(&Url::new(url)?.pathname());
            self.mark_active_navigation()?;
            self.set_navigation(false)?;
            if let Ok(main) = current_main.dyn_into::<HtmlElement>() {
                main.focus()?;
            }
            Ok(())
        }

        fn apply_theme(&self) -> Result<(), JsValue> {
            let theme = self.state.borrow().theme;
            let root = self
                .document
                .document_element()
                .ok_or_else(|| JsValue::from_str("document root is missing"))?;
            root.set_attribute("data-theme", theme.as_str())?;
            if let Some(button) = self.document.query_selector("[data-theme-toggle]")? {
                let next = if theme == Theme::Dark {
                    "light"
                } else {
                    "dark"
                };
                button.set_attribute("aria-label", &format!("Switch to {next} theme"))?;
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

        fn set_navigation(&self, open: bool) -> Result<(), JsValue> {
            let was_open = self.state.borrow().navigation_open;
            self.state.borrow_mut().navigation_open = open;
            let root = self
                .document
                .document_element()
                .ok_or_else(|| JsValue::from_str("document root is missing"))?;
            root.set_attribute("data-navigation", if open { "open" } else { "closed" })?;
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
                if !open
                    && was_open
                    && let Ok(button) = button.dyn_into::<HtmlElement>()
                {
                    let _ = button.focus();
                }
            }
            Ok(())
        }

        fn set_navigation_status(&self, status: NavigationStatus) -> Result<(), JsValue> {
            let root = self
                .document
                .document_element()
                .ok_or_else(|| JsValue::from_str("document root is missing"))?;
            root.set_attribute(
                "data-navigation-status",
                if status == NavigationStatus::Loading {
                    "loading"
                } else {
                    "idle"
                },
            )
        }

        fn mark_active_navigation(&self) -> Result<(), JsValue> {
            let active_route = self.state.borrow().route.as_str();
            let links = self.document.query_selector_all(".site-nav [data-route]")?;
            for index in 0..links.length() {
                if let Some(element) = links
                    .item(index)
                    .and_then(|node| node.dyn_into::<Element>().ok())
                {
                    if element.get_attribute("data-route").as_deref() == Some(active_route) {
                        element.set_attribute("aria-current", "page")?;
                    } else {
                        element.remove_attribute("aria-current")?;
                    }
                }
            }
            Ok(())
        }

        fn scroll_to_fragment(&self, hash: &str) -> bool {
            let id = hash.strip_prefix('#').unwrap_or(hash);
            if id.is_empty() {
                return false;
            }
            let Some(target) = self.document.get_element_by_id(id) else {
                return false;
            };
            let top =
                target.get_bounding_client_rect().top() + self.window.scroll_y().unwrap_or(0.0);
            let options = ScrollToOptions::new();
            options.set_top(top);
            options.set_behavior(if prefers_reduced_motion(&self.window) {
                ScrollBehavior::Auto
            } else {
                ScrollBehavior::Smooth
            });
            self.window.scroll_to_with_scroll_to_options(&options);
            true
        }
    }

    fn is_unmodified_primary_click(event: &MouseEvent) -> bool {
        event.button() == 0
            && !event.alt_key()
            && !event.ctrl_key()
            && !event.meta_key()
            && !event.shift_key()
    }

    fn persist_theme(window: &Window, document: &Document, theme: Theme) -> Result<(), JsValue> {
        let hostname = window.location().hostname()?;
        let local = matches!(hostname.as_str(), "localhost" | "127.0.0.1" | "::1");
        let html_document: HtmlDocument = document.clone().unchecked_into();
        html_document.set_cookie(&ThemeStore::cookie(theme, !local))
    }

    fn prefers_reduced_motion(window: &Window) -> bool {
        window
            .match_media("(prefers-reduced-motion: reduce)")
            .ok()
            .flatten()
            .is_some_and(|query| query.matches())
    }
}

#[cfg(test)]
mod tests {
    use super::{AppState, Route, Theme, ThemeStore};

    #[test]
    fn cookie_precedes_legacy_storage() {
        let resolution = ThemeStore::resolve("session=x; theme=light", Some("dark"));
        assert_eq!(resolution.theme, Theme::Light);
        assert!(!resolution.migrated_legacy_value);
    }

    #[test]
    fn valid_legacy_storage_migrates_once() {
        let resolution = ThemeStore::resolve("session=x", Some("light"));
        assert_eq!(resolution.theme, Theme::Light);
        assert!(resolution.migrated_legacy_value);
    }

    #[test]
    fn invalid_values_retain_dark_default() {
        assert_eq!(
            ThemeStore::resolve("theme=sepia", Some("blue")).theme,
            Theme::Dark
        );
    }

    #[test]
    fn cookie_has_functional_security_attributes() {
        assert_eq!(
            ThemeStore::cookie(Theme::Dark, true),
            "theme=dark; Path=/; Max-Age=31536000; SameSite=Lax; Secure"
        );
    }

    #[test]
    fn route_set_is_bounded() {
        assert_eq!(Route::from_path("/notes/"), Route::Notes);
        assert_eq!(Route::from_path("/other/"), Route::NotFound);
    }

    #[test]
    fn request_generation_and_scroll_state_are_deterministic() {
        let mut state = AppState::new(Theme::Dark, Route::Home);
        state.remember_scroll("https://example.test/".into(), 120.5);
        let first = state.begin_request();
        let second = state.begin_request();
        assert!(!state.owns_request(first));
        assert!(state.owns_request(second));
        assert_eq!(state.scroll_for("https://example.test/"), 120.5);
        assert_eq!(state.scroll_for("https://example.test/new"), 0.0);
    }
}
