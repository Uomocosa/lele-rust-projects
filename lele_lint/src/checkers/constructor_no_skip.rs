use crate::checker::Checker;
use crate::config::Config;

use super::constructor_no_skip_check;
use super::constructor_no_skip_code;
use super::constructor_no_skip_name;
use super::constructor_no_skip_register;

pub struct ConstructorNoSkip;

#[rustfmt::skip]
impl Checker for ConstructorNoSkip {
    fn name(&self) -> &'static str { constructor_no_skip_name::name(self) }
    fn code(&self) -> &'static str { constructor_no_skip_code::code(self) }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { constructor_no_skip_check::check(self, project) }
}

#[rustfmt::skip]
impl ConstructorNoSkip {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        constructor_no_skip_register::register(checkers, config)
    }
}

// no test_usage necessary
