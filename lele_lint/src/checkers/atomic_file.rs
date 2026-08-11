use crate::checker;
use crate::config;
use crate::diagnostic;
use crate::project;

use super::atomic_file_check;
use super::atomic_file_register;

pub struct AtomicFile;

impl AtomicFile {
    pub const NAME: &'static str = "atomic_file";
    pub const CODE: &'static str = "E001";
}

#[rustfmt::skip]
impl checker::Checker for AtomicFile {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &project::Project) -> Vec<diagnostic::Diagnostic> { atomic_file_check::check(self, project) }
}

#[rustfmt::skip]
impl AtomicFile {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        atomic_file_register::register(checkers, config)
    }
}

// no test_usage necessary
