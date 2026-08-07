use crate::checker::Checker;
use crate::config::Config;

use super::domain_import_check;
use super::domain_import_register;

pub struct DomainImport;

impl DomainImport {
    pub const NAME: &'static str = "domain_import";
    pub const CODE: &'static str = "E011";
}

#[rustfmt::skip]
impl Checker for DomainImport {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { domain_import_check::check(self, project) }
}

#[rustfmt::skip]
impl DomainImport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        domain_import_register::register(checkers, config)
    }
}

// no test_usage necessary
