use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::atomic_file_check;
use super::atomic_file_register;

pub struct AtomicFile;

impl AtomicFile {
    pub const NAME: &'static str = "atomic_file";
    pub const CODE: &'static str = "E001";
}

#[rustfmt::skip]
impl Checker for AtomicFile {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { atomic_file_check::check(self, project) }
}

#[rustfmt::skip]
impl AtomicFile {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        atomic_file_register::register(checkers, config)
    }
}

// no test_usage necessary
