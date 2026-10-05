//! Theme preference, section links, email contact, and citation copying.
mod theme;
pub use theme::Theme;

#[cfg(target_arch = "wasm32")]
mod browser;
