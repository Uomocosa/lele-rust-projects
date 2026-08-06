use crate::checker::Checker;
use crate::config::Config;

use super::test_inline_check;
use super::test_inline_code;
use super::test_inline_name;
use super::test_inline_register;

pub struct TestInline;

#[rustfmt::skip]
impl Checker for TestInline {
    fn name(&self) -> &'static str { test_inline_name::name(self) }
    fn code(&self) -> &'static str { test_inline_code::code(self) }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { test_inline_check::check(self, project) }
}

#[rustfmt::skip]
impl TestInline {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        test_inline_register::register(checkers, config)
    }
}

// no test_usage necessary
