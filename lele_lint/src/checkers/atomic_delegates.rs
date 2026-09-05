use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::atomic_delegates_check;
use super::atomic_delegates_register;

pub struct AtomicDelegates;

impl AtomicDelegates {
    pub const NAME: &'static str = "atomic_delegates";
    pub const CODE: &'static str = "E012";
}

#[rustfmt::skip]
impl Checker for AtomicDelegates {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { atomic_delegates_check::check(self, project) }
}

#[rustfmt::skip]
impl AtomicDelegates {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        atomic_delegates_register::register(checkers, config)
    }
}

// no test_usage necessary
