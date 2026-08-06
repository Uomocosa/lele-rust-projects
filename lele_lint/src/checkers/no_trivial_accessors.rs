use crate::checker::Checker;
use crate::config::Config;

use super::no_trivial_accessors_check;
use super::no_trivial_accessors_code;
use super::no_trivial_accessors_name;
use super::no_trivial_accessors_register;

pub struct NoTrivialAccessors;

#[rustfmt::skip]
impl Checker for NoTrivialAccessors {
    fn name(&self) -> &'static str { no_trivial_accessors_name::name(self) }
    fn code(&self) -> &'static str { no_trivial_accessors_code::code(self) }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { no_trivial_accessors_check::check(self, project) }
}

#[rustfmt::skip]
impl NoTrivialAccessors {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        no_trivial_accessors_register::register(checkers, config)
    }
}

// no test_usage necessary
