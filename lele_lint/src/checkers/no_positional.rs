use crate::checker::Checker;
use crate::config::Config;

use super::no_positional_check;
use super::no_positional_register;

pub struct NoPositional;

impl NoPositional {
    pub const NAME: &'static str = "no_positional";
    pub const CODE: &'static str = "E009";
}

#[rustfmt::skip]
impl Checker for NoPositional {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { no_positional_check::check(self, project) }
}

#[rustfmt::skip]
impl NoPositional {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        no_positional_register::register(checkers, config)
    }
}

// no test_usage necessary
