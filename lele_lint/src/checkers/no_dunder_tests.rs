use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::no_dunder_tests_check;
use super::no_dunder_tests_register;

pub struct NoDunderTests;

impl NoDunderTests {
    pub const NAME: &'static str = "no_dunder_tests";
    pub const CODE: &'static str = "E034";
}

#[rustfmt::skip]
impl Checker for NoDunderTests {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { no_dunder_tests_check::check(self, project) }
}

#[rustfmt::skip]
impl NoDunderTests {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        no_dunder_tests_register::register(checkers, config)
    }
}

// no test_usage necessary
