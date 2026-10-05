//! Filter the existing publication cards without storing or sending queries.

use super::{App, EVENT_HANDLERS, EventHandler};
use crate::publications::Query;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use web_sys::{Element, Event, HtmlInputElement};

impl App {
    pub(super) fn enable_publication_filter(&self) -> Result<(), JsValue> {
        let Some(filter) = self.document.query_selector(".publication-filter")? else {
            return Ok(());
        };
        let input = self
            .document
            .get_element_by_id("publication-search")
            .ok_or("publication search is unavailable")?
            .dyn_into::<HtmlInputElement>()?;
        let count = self
            .document
            .get_element_by_id("publication-count")
            .ok_or("publication count is unavailable")?;
        let empty = self
            .document
            .get_element_by_id("publication-empty")
            .ok_or("publication empty state is unavailable")?;
        let cards = self
            .document
            .query_selector_all(".publication-list > .paper")?;
        let mut publications = Vec::new();
        for index in 0..cards.length() {
            let card = cards
                .item(index)
                .ok_or("publication card is unavailable")?
                .dyn_into::<Element>()?;
            let mut fields = Vec::new();
            for selector in ["h3", ".authors", ".meta"] {
                if let Some(field) = card.query_selector(selector)? {
                    fields.push(field.text_content().unwrap_or_default());
                }
            }
            publications.push((card, fields.join("\n")));
        }

        update(&publications, &count, &empty, &input.value())?;
        let search_input = input.clone();
        let search: EventHandler = Closure::new(move |_: Event| {
            let _ = update(&publications, &count, &empty, &search_input.value());
        });
        input.add_event_listener_with_callback("input", search.as_ref().unchecked_ref())?;
        EVENT_HANDLERS.with(|handlers| handlers.borrow_mut().push(search));
        filter.remove_attribute("hidden")?;
        Ok(())
    }
}

fn update(
    publications: &[(Element, String)],
    count: &Element,
    empty: &Element,
    input: &str,
) -> Result<(), JsValue> {
    let query = Query::new(input);
    let mut visible = 0;
    for (card, text) in publications {
        if query.matches(text) {
            card.remove_attribute("hidden")?;
            visible += 1;
        } else {
            card.set_attribute("hidden", "")?;
        }
    }

    let total = publications.len();
    let noun = if total == 1 {
        "publication"
    } else {
        "publications"
    };
    let label = if visible == total {
        format!("{total} {noun}")
    } else {
        format!("{visible} of {total} {noun}")
    };
    count.set_text_content(Some(&label));
    if visible == 0 {
        empty.remove_attribute("hidden")?;
    } else {
        empty.set_attribute("hidden", "")?;
    }
    Ok(())
}
