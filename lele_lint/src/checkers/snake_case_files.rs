use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct SnakeCaseFiles;

impl SnakeCaseFiles {
    pub const NAME: &'static str = "snake_case_files";
    pub const CODE: &'static str = "E002";
}

#[rustfmt::skip]
impl Checker for SnakeCaseFiles {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(SnakeCaseFiles)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl SnakeCaseFiles {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(SnakeCaseFiles));
    }
}

// no test_usage necessary
