use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, Document, Element, Event};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let Some(win) = window() else {
        return Ok(());
    };
    let Some(document) = win.document() else {
        return Ok(());
    };

    if let Some(body) = document.body() {
        let _ = body.set_attribute("data-wasm", "ready");
    }

    apply_stored_theme(&document);
    mark_active_nav(&document);
    install_theme_toggle(&document)?;
    Ok(())
}

fn mark_active_nav(document: &Document) {
    let Some(win) = window() else {
        return;
    };
    let path = win.location().pathname().unwrap_or_default();
    let route = if path.starts_with("/publications") {
        "publications"
    } else if path.starts_with("/resources") {
        "resources"
    } else {
        "home"
    };

    let selector = format!("a[data-route='{route}']");
    if let Ok(Some(active)) = document.query_selector(&selector) {
        let _ = active.set_attribute("aria-current", "page");
    }
}

fn install_theme_toggle(document: &Document) -> Result<(), JsValue> {
    let Some(button) = query(document, "[data-theme-toggle]") else {
        return Ok(());
    };

    update_theme_toggle_label(document);

    let doc = document.clone();
    let closure = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_event: Event| {
        toggle_theme(&doc);
    }));

    button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn apply_stored_theme(document: &Document) {
    match stored_theme().as_deref() {
        Some("light") => set_document_theme(document, "light"),
        Some("dark") => set_document_theme(document, "dark"),
        _ => clear_document_theme(document),
    }
    update_theme_toggle_label(document);
}

fn toggle_theme(document: &Document) {
    let current = effective_theme(document);
    let next = if current == "dark" { "light" } else { "dark" };
    set_document_theme(document, next);
    store_theme(next);
    update_theme_toggle_label(document);
}

fn set_document_theme(document: &Document, theme: &str) {
    if let Some(root) = document.document_element() {
        let _ = root.set_attribute("data-theme", theme);
    }
}

fn clear_document_theme(document: &Document) {
    if let Some(root) = document.document_element() {
        let _ = root.remove_attribute("data-theme");
    }
}

fn effective_theme(document: &Document) -> &'static str {
    if let Some(root) = document.document_element() {
        if let Some(theme) = root.get_attribute("data-theme") {
            if theme == "light" {
                return "light";
            }
            if theme == "dark" {
                return "dark";
            }
        }
    }

    if prefers_dark() {
        "dark"
    } else {
        "light"
    }
}

fn update_theme_toggle_label(document: &Document) {
    let current = effective_theme(document);
    let next = if current == "dark" { "Light" } else { "Dark" };

    if let Some(button) = query(document, "[data-theme-toggle]") {
        let _ = button.set_attribute("aria-label", &format!("Switch to {next} theme"));
        let _ = button.set_attribute(
            "aria-pressed",
            if current == "dark" { "true" } else { "false" },
        );
    }

    if let Some(label) = query(document, "[data-theme-label]") {
        label.set_text_content(Some(next));
    }
}

fn query(document: &Document, selector: &str) -> Option<Element> {
    document.query_selector(selector).ok().flatten()
}

fn stored_theme() -> Option<String> {
    window()
        .and_then(|win| win.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item("theme").ok().flatten())
}

fn store_theme(theme: &str) {
    if let Some(storage) = window().and_then(|win| win.local_storage().ok().flatten()) {
        let _ = storage.set_item("theme", theme);
    }
}

fn prefers_dark() -> bool {
    window()
        .and_then(|win| {
            win.match_media("(prefers-color-scheme: dark)")
                .ok()
                .flatten()
        })
        .map(|query| query.matches())
        .unwrap_or(false)
}
