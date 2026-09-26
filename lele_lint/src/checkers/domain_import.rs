use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct DomainImport;

impl DomainImport {
    pub const NAME: &'static str = "domain_import";
    pub const CODE: &'static str = "E011";
}

#[rustfmt::skip]
impl Checker for DomainImport {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::domain_import_check::check(self, project) }
}

#[atomic_delegates]
impl DomainImport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
