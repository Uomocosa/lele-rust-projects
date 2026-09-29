mod highlighter_for_extension;
pub use highlighter_for_extension::highlighter_for_extension;
mod highlighter_new;
pub use highlighter_new::highlighter_new;
mod render_html;
pub use render_html::render_html;

#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::structs::Highlighter;
