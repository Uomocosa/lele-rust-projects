#![allow(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::string_slice,
    clippy::unwrap_used
)]
pub mod checker;
pub mod checkers;
mod common;
pub mod config;
mod config_checker_enabled;
mod config_load;
pub mod diagnostic;
pub mod entry;
pub mod entry_kind;
pub mod error;
pub mod lele_lint_section;
pub mod mod_decl;
pub mod module_info;
mod module_info_build;
pub mod print_checker_list;
pub mod print_diagnostics;
pub mod project;
mod project_discover;
mod project_find_cargo_root;
mod project_get_parsed;
mod project_parse_source_files;
mod project_walk_entries;
pub mod reexport;
pub mod severity;
