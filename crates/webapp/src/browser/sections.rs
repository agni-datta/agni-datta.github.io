//! Native section fragments and the References default-folding policy.
use super::App;
use wasm_bindgen::JsValue;

impl App {
    pub(super) fn fold_references(&self) -> Result<(), JsValue> {
        if let Some(section) = self.document.get_element_by_id("references") {
            section.remove_attribute("open")?;
        }
        Ok(())
    }

    pub(super) fn reveal_linked_section(&self) -> Result<(), JsValue> {
        if let Some(location) = self.document.location()
            && let Some(id) = location.hash()?.strip_prefix('#')
        {
            self.reveal_section(id)?;
        }
        Ok(())
    }

    fn reveal_section(&self, id: &str) -> Result<(), JsValue> {
        let mut element = self.document.get_element_by_id(id);
        while let Some(section) = element {
            if section.get_attribute("id").as_deref() == Some("references") {
                break;
            }
            if section.tag_name() == "DETAILS" {
                section.set_attribute("open", "")?;
            }
            element = section.parent_element();
        }
        Ok(())
    }
}
