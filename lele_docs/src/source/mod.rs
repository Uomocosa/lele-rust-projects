mod highlighter_new;
pub use highlighter_new::highlighter_new;
mod render_html;
pub use render_html::render_html;

#[path = "__basic__/mod.rs"]
pub mod basic;
pub use basic::structs::Highlighter;
