use crate::checkers;
use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

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

#[rustfmt::skip]
impl DomainImport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        checkers::domain_import_register::register(checkers, config)
    }
}

// no test_usage necessary
