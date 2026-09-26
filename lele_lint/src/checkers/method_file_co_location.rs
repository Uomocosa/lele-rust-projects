use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct MethodFileCoLocation;

impl MethodFileCoLocation {
    pub const NAME: &'static str = "method_file_co_location";
    pub const CODE: &'static str = "E017";
}

#[rustfmt::skip]
impl Checker for MethodFileCoLocation {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::method_file_co_location_check::check(self, project) }
}

#[atomic_delegates]
impl MethodFileCoLocation {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
