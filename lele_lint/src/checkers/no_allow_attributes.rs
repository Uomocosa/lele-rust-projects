use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::no_allow_attributes_check;
use super::no_allow_attributes_register;

pub struct NoAllowAttributes;

impl NoAllowAttributes {
    pub const NAME: &'static str = "no_allow_attributes";
    pub const CODE: &'static str = "E023";
}

#[rustfmt::skip]
impl Checker for NoAllowAttributes {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { no_allow_attributes_check::check(self, project) }
}

#[rustfmt::skip]
impl NoAllowAttributes {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        no_allow_attributes_register::register(checkers, config)
    }
}

// no test_usage necessary
