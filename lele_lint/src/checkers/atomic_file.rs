use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct AtomicFile;

impl AtomicFile {
    pub const NAME: &'static str = "atomic_file";
    pub const CODE: &'static str = "E001";
}

#[rustfmt::skip]
impl Checker for AtomicFile {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::atomic_file_check::check(self, project) }
}

#[atomic_delegates]
impl AtomicFile {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
