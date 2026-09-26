use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct NoCrossDomainReexport;

impl NoCrossDomainReexport {
    pub const NAME: &'static str = "no_cross_domain_reexport";
    pub const CODE: &'static str = "E004";
}

#[rustfmt::skip]
impl Checker for NoCrossDomainReexport {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::no_cross_domain_reexport_check::check(self, project) }
}

#[atomic_delegates]
impl NoCrossDomainReexport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
