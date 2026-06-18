use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, Document, Element, Event, ScrollBehavior, ScrollToOptions};

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
    install_smooth_anchor_scroll(&document)?;
    Ok(())
}

fn mark_active_nav(document: &Document) {
    let Some(win) = window() else {
        return;
    };
    let path = win.location().pathname().unwrap_or_default();
    let route = if path.starts_with("/publications") {
        "publications"
    } else if path.starts_with("/notes") {
        "notes"
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

fn install_smooth_anchor_scroll(document: &Document) -> Result<(), JsValue> {
    if prefers_reduced_motion() {
        return Ok(());
    }

    let links = document.query_selector_all("a[href^='#']")?;
    for index in 0..links.length() {
        let Some(node) = links.item(index) else {
            continue;
        };
        let Ok(link) = node.dyn_into::<Element>() else {
            continue;
        };

        let doc = document.clone();
        let link_for_handler = link.clone();
        let closure = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |event: Event| {
            let Some(href) = link_for_handler.get_attribute("href") else {
                return;
            };
            if href.len() <= 1 {
                return;
            }

            let selector = css_id_selector(&href[1..]);
            let Ok(Some(target)) = doc.query_selector(&selector) else {
                return;
            };

            event.prevent_default();
            smooth_scroll_to(&target);
        }));

        link.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    Ok(())
}

fn apply_stored_theme(document: &Document) {
    match stored_theme().as_deref() {
        Some("light") => set_document_theme(document, "light"),
        Some("dark") => set_document_theme(document, "dark"),
        _ => set_document_theme(document, "dark"),
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
    "dark"
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

fn prefers_reduced_motion() -> bool {
    window()
        .and_then(|win| {
            win.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .map(|query| query.matches())
        .unwrap_or(false)
}

fn smooth_scroll_to(target: &Element) {
    let Some(win) = window() else {
        return;
    };
    let top = target.get_bounding_client_rect().top() + win.scroll_y().unwrap_or(0.0);
    let options = ScrollToOptions::new();
    options.set_top(top);
    options.set_behavior(ScrollBehavior::Smooth);
    win.scroll_to_with_scroll_to_options(&options);
}

fn css_id_selector(id: &str) -> String {
    let escaped = id.replace('\\', "\\\\").replace('\'', "\\'");
    format!("#{escaped}")
}
