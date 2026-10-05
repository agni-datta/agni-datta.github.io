//! Decode the contact token only after an explicit visitor click.
use super::App;
use wasm_bindgen::JsValue;
use web_sys::Element;

impl App {
    pub(super) fn enable_contact(&self) -> Result<(), JsValue> {
        if let Some(button) = self.document.query_selector("[data-email-token]")? {
            button.remove_attribute("disabled")?;
        }
        Ok(())
    }

    pub(super) fn open_contact(&self, button: Element) -> Result<(), JsValue> {
        let token = button.get_attribute("data-email-token").unwrap_or_default();
        let window = web_sys::window().ok_or("window is unavailable")?;
        // Reversible obfuscation: decode only for an explicit contact action.
        // Keep the address out of the page text and persistent DOM attributes.
        let address = window.atob(&token)?;
        window.open_with_url_and_target(&format!("mailto:{address}"), "_self")?;
        Ok(())
    }
}
