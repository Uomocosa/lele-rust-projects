use crate::checker;
use crate::config;
use crate::diagnostic;
use crate::project;

use super::test_usage_check;
use super::test_usage_register;

pub struct TestUsage;

impl TestUsage {
    pub const NAME: &'static str = "test_usage";
    pub const CODE: &'static str = "E006";
}

#[rustfmt::skip]
impl checker::Checker for TestUsage {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &project::Project) -> Vec<diagnostic::Diagnostic> { test_usage_check::check(self, project) }
}

#[rustfmt::skip]
impl TestUsage {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        test_usage_register::register(checkers, config)
    }
}

// no test_usage necessary
