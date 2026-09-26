use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct AtomicFile;

impl AtomicFile {
    pub const NAME: &'static str = "atomic_file";
    pub const CODE: &'static str = "E001";
}

#[rustfmt::skip]
impl Checker for AtomicFile {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(AtomicFile)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl AtomicFile {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(AtomicFile));
    }
}

// no test_usage necessary
