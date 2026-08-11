use crate::checker;
use crate::config;
use crate::diagnostic;
use crate::project;

use super::domain_import_check;
use super::domain_import_register;

pub struct DomainImport;

impl DomainImport {
    pub const NAME: &'static str = "domain_import";
    pub const CODE: &'static str = "E011";
}

#[rustfmt::skip]
impl checker::Checker for DomainImport {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &project::Project) -> Vec<diagnostic::Diagnostic> { domain_import_check::check(self, project) }
}

#[rustfmt::skip]
impl DomainImport {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        domain_import_register::register(checkers, config)
    }
}

// no test_usage necessary
