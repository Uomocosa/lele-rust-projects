use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct HelperCount;

impl HelperCount {
    pub const NAME: &'static str = "helper_count";
    pub const CODE: &'static str = "E015";
}

#[rustfmt::skip]
impl Checker for HelperCount {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::helper_count_check::check(self, project) }
}

#[atomic_delegates]
impl HelperCount {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
