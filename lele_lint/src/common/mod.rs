mod has_rustfmt_skip;
mod is_cfg_test_mod;
mod is_default_impl;
mod is_delegate_call;
mod is_short_body;
mod is_stuttered_path;
mod primary_type_name;
mod self_type_last;
mod to_pascal_case;

mod collect_declared;
mod comment_scan;
mod file_cfgs;
mod has_atomic_delegate;
mod module_cfgs;
mod module_paths;
mod root_index_content;
mod type_index_content;

#[path = "__basic__/mod.rs"]
pub mod basic;

pub(crate) use basic::structs::{CommentHit, DeclaredType};
pub(crate) use basic::type_aliases::ModuleCfgMap;

pub(crate) use to_pascal_case::to_pascal_case;

pub(crate) use collect_declared::collect_declared;
pub(crate) use comment_scan::find_comments;
pub(crate) use file_cfgs::file_cfgs;
pub(crate) use has_atomic_delegate::has_atomic_delegate;
pub(crate) use has_rustfmt_skip::has_rustfmt_skip;
pub(crate) use is_cfg_test_mod::is_cfg_test_mod;
pub(crate) use is_default_impl::is_default_impl;
pub(crate) use is_delegate_call::is_delegate_call;
pub(crate) use is_short_body::is_short_body;
pub(crate) use is_stuttered_path::is_stuttered_path;
pub(crate) use module_paths::module_path_of;
pub(crate) use primary_type_name::primary_type_name;
pub(crate) use root_index_content::root_index_content;
pub(crate) use self_type_last::self_type_last;
pub(crate) use type_index_content::type_index_content;

pub(crate) use lele_snake_case::to_snake_case;

// no test_usage necessary
