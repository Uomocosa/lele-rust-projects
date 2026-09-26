use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct NoCrossDomainReexport;

impl NoCrossDomainReexport {
    pub const NAME: &'static str = "no_cross_domain_reexport";
    pub const CODE: &'static str = "E004";
}

#[rustfmt::skip]
impl Checker for NoCrossDomainReexport {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(NoCrossDomainReexport)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoCrossDomainReexport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoCrossDomainReexport));
    }
}

// no test_usage necessary
