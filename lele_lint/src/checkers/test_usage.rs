use crate::checker::Checker;
use crate::config::Config;

use super::test_usage_check;
use super::test_usage_code;
use super::test_usage_name;
use super::test_usage_register;

pub struct TestUsage;

#[rustfmt::skip]
impl Checker for TestUsage {
    fn name(&self) -> &'static str { test_usage_name::name(self) }
    fn code(&self) -> &'static str { test_usage_code::code(self) }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { test_usage_check::check(self, project) }
}

#[rustfmt::skip]
impl TestUsage {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        test_usage_register::register(checkers, config)
    }
}

// no test_usage necessary
