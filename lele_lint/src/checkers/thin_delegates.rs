use crate::checker::Checker;
use crate::config::Config;

use super::thin_delegates_check;
use super::thin_delegates_register;

pub struct ThinDelegates;

impl ThinDelegates {
    pub const NAME: &'static str = "thin_delegates";
    pub const CODE: &'static str = "E012";
}

#[rustfmt::skip]
impl Checker for ThinDelegates {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { thin_delegates_check::check(self, project) }
}

#[rustfmt::skip]
impl ThinDelegates {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        thin_delegates_register::register(checkers, config)
    }
}

// no test_usage necessary
