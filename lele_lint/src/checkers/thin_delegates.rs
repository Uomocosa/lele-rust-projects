use crate::checker;
use crate::config;
use crate::diagnostic;
use crate::project;

use super::thin_delegates_check;
use super::thin_delegates_register;

pub struct ThinDelegates;

impl ThinDelegates {
    pub const NAME: &'static str = "thin_delegates";
    pub const CODE: &'static str = "E012";
}

#[rustfmt::skip]
impl checker::Checker for ThinDelegates {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &project::Project) -> Vec<diagnostic::Diagnostic> { thin_delegates_check::check(self, project) }
}

#[rustfmt::skip]
impl ThinDelegates {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        thin_delegates_register::register(checkers, config)
    }
}

// no test_usage necessary
