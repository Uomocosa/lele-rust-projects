// no test_usage necessary

use crate::checker::Checker;
use crate::config::Config;

use super::thin_delegates_check;
use super::thin_delegates_meta;
use super::thin_delegates_register;

pub struct ThinDelegates;

#[rustfmt::skip]
impl Checker for ThinDelegates {
    fn name(&self) -> &'static str { thin_delegates_meta::name(self) }
    fn code(&self) -> &'static str { thin_delegates_meta::code(self) }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { thin_delegates_check::check(self, project) }
}

#[rustfmt::skip]
impl ThinDelegates {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        thin_delegates_register::register(checkers, config)
    }
}
