//! Theme preference, publication filtering, section links, contact, and citation copying.
mod theme;
pub use theme::Theme;

#[cfg(any(target_arch = "wasm32", test))]
mod publications;

#[cfg(target_arch = "wasm32")]
mod browser;
