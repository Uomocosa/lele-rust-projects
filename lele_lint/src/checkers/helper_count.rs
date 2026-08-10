use crate::checker;
use crate::config;

use super::helper_count_check;
use super::helper_count_register;

pub struct HelperCount;

impl HelperCount {
    pub const NAME: &'static str = "helper_count";
    pub const CODE: &'static str = "E015";
}

#[rustfmt::skip]
impl checker::Checker for HelperCount {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { helper_count_check::check(self, project) }
}

#[rustfmt::skip]
impl HelperCount {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        helper_count_register::register(checkers, config)
    }
}

// no test_usage necessary
