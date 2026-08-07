use crate::checker::Checker;
use crate::config::Config;

use super::test_usage_check;
use super::test_usage_register;

pub struct TestUsage;

impl TestUsage {
    pub const NAME: &'static str = "test_usage";
    pub const CODE: &'static str = "E006";
}

#[rustfmt::skip]
impl Checker for TestUsage {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { test_usage_check::check(self, project) }
}

#[rustfmt::skip]
impl TestUsage {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        test_usage_register::register(checkers, config)
    }
}

// no test_usage necessary
