//! Browser lifecycle and explicit click dispatch.
use std::cell::RefCell;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use web_sys::{Document, Element, Event};

mod citations;
mod contact;
mod publications;
mod sections;
mod theme;

type EventHandler = Closure<dyn FnMut(Event)>;

// Keep callbacks alive for this document without storing visitor state.
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
    app.fold_references()?;
    app.reveal_linked_section()?;
    app.enable_publication_filter()?;

    let click_app = app.clone();
    let click: EventHandler = Closure::new(move |event: Event| {
        let _ = click_app.click(&event);
    });
    app.document
        .add_event_listener_with_callback("click", click.as_ref().unchecked_ref())?;

    app.enable_contact()?;

    let hash_app = app.clone();
    let hash: EventHandler = Closure::new(move |_: Event| {
        let _ = hash_app.reveal_linked_section();
    });
    window.add_event_listener_with_callback("hashchange", hash.as_ref().unchecked_ref())?;

    // Back/forward can restore a cached document whose theme is stale.
    let page: EventHandler = Closure::new(move |_: Event| {
        let _ = app.restore_theme();
        let _ = app.fold_references();
    });
    window.add_event_listener_with_callback("pageshow", page.as_ref().unchecked_ref())?;
    EVENT_HANDLERS.with(|handlers| handlers.borrow_mut().extend([click, hash, page]));
    Ok(())
}

impl App {
    fn click(&self, event: &Event) -> Result<(), JsValue> {
        let Some(target) = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
        else {
            return Ok(());
        };
        if target.closest("[data-theme-toggle]")?.is_some() {
            self.toggle_theme()?;
        } else if let Some(button) = target.closest("[data-copy-citation]")? {
            self.copy_citation(button)?;
        } else if let Some(button) = target.closest("[data-email-token]")? {
            self.open_contact(button)?;
        }
        Ok(())
    }
}
