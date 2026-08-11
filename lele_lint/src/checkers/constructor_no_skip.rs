use crate::checker;
use crate::config;
use crate::diagnostic;
use crate::project;

use super::constructor_no_skip_check;
use super::constructor_no_skip_register;

pub struct ConstructorNoSkip;

impl ConstructorNoSkip {
    pub const NAME: &'static str = "constructor_no_skip";
    pub const CODE: &'static str = "E013";
}

#[rustfmt::skip]
impl checker::Checker for ConstructorNoSkip {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &project::Project) -> Vec<diagnostic::Diagnostic> { constructor_no_skip_check::check(self, project) }
}

#[rustfmt::skip]
impl ConstructorNoSkip {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        constructor_no_skip_register::register(checkers, config)
    }
}

// no test_usage necessary
