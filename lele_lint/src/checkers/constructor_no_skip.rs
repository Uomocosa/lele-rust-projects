use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::constructor_no_skip_check;
use super::constructor_no_skip_register;

pub struct ConstructorNoSkip;

impl ConstructorNoSkip {
    pub const NAME: &'static str = "constructor_no_skip";
    pub const CODE: &'static str = "E013";
}

#[rustfmt::skip]
impl Checker for ConstructorNoSkip {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { constructor_no_skip_check::check(self, project) }
}

#[rustfmt::skip]
impl ConstructorNoSkip {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        constructor_no_skip_register::register(checkers, config)
    }
}

// no test_usage necessary
