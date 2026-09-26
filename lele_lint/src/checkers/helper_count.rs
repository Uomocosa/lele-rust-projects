use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct HelperCount;

impl HelperCount {
    pub const NAME: &'static str = "helper_count";
    pub const CODE: &'static str = "E015";
}

#[rustfmt::skip]
impl Checker for HelperCount {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(HelperCount)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl HelperCount {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(HelperCount));
    }
}

// no test_usage necessary
