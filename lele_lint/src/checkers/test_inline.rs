use crate::checker::Checker;
use crate::config::Config;

use super::test_inline_check;
use super::test_inline_register;

pub struct TestInline;

impl TestInline {
    pub const NAME: &'static str = "test_inline";
    pub const CODE: &'static str = "E007";
}

#[rustfmt::skip]
impl Checker for TestInline {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { test_inline_check::check(self, project) }
}

#[rustfmt::skip]
impl TestInline {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        test_inline_register::register(checkers, config)
    }
}

// no test_usage necessary
