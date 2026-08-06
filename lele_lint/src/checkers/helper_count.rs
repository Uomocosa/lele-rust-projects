use crate::checker::Checker;
use crate::config::Config;

use super::helper_count_check;
use super::helper_count_code;
use super::helper_count_name;
use super::helper_count_register;

pub struct HelperCount;

#[rustfmt::skip]
impl Checker for HelperCount {
    fn name(&self) -> &'static str { helper_count_name::name(self) }
    fn code(&self) -> &'static str { helper_count_code::code(self) }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { helper_count_check::check(self, project) }
}

#[rustfmt::skip]
impl HelperCount {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        helper_count_register::register(checkers, config)
    }
}

// no test_usage necessary
