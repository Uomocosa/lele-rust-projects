// no test_usage necessary

use crate::checker::Checker;
use crate::config::Config;

use super::atomic_file_check;
use super::atomic_file_meta;
use super::atomic_file_register;

pub struct AtomicFile;

#[rustfmt::skip]
impl Checker for AtomicFile {
    fn name(&self) -> &'static str { atomic_file_meta::name(self) }
    fn code(&self) -> &'static str { atomic_file_meta::code(self) }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { atomic_file_check::check(self, project) }
}

#[rustfmt::skip]
impl AtomicFile {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        atomic_file_register::register(checkers, config)
    }
}
