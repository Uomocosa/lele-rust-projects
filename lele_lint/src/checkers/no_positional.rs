// no test_usage necessary

use crate::checker::Checker;
use crate::config::Config;

use super::no_positional_check;
use super::no_positional_meta;
use super::no_positional_register;

pub struct NoPositional;

#[rustfmt::skip]
impl Checker for NoPositional {
    fn name(&self) -> &'static str { no_positional_meta::name(self) }
    fn code(&self) -> &'static str { no_positional_meta::code(self) }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { no_positional_check::check(self, project) }
}

#[rustfmt::skip]
impl NoPositional {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        no_positional_register::register(checkers, config)
    }
}
