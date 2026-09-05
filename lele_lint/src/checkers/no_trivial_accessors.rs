use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::no_trivial_accessors_check;
use super::no_trivial_accessors_register;

pub struct NoTrivialAccessors;

impl NoTrivialAccessors {
    pub const NAME: &'static str = "no_trivial_accessors";
    pub const CODE: &'static str = "E010";
}

#[rustfmt::skip]
impl Checker for NoTrivialAccessors {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { no_trivial_accessors_check::check(self, project) }
}

#[rustfmt::skip]
impl NoTrivialAccessors {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        no_trivial_accessors_register::register(checkers, config)
    }
}

// no test_usage necessary
