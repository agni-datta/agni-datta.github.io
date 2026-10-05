//! User-triggered, write-only citation copying.
use super::App;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::Element;

impl App {
    pub(super) fn copy_citation(&self, button: Element) -> Result<(), JsValue> {
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
}
