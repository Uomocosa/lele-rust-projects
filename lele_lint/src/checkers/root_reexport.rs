use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct RootReexport;

impl RootReexport {
    pub const NAME: &'static str = "root_reexport";
    pub const CODE: &'static str = "E024";
}

#[rustfmt::skip]
impl Checker for RootReexport {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(RootReexport)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl RootReexport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(RootReexport));
    }
}

// no test_usage necessary
