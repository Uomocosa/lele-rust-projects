// no test_usage necessary

use crate::checker::Checker;
use crate::config::Config;

use super::domain_import_check;
use super::domain_import_meta;
use super::domain_import_register;

pub struct DomainImport;

#[rustfmt::skip]
impl Checker for DomainImport {
    fn name(&self) -> &'static str { domain_import_meta::name(self) }
    fn code(&self) -> &'static str { domain_import_meta::code(self) }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { domain_import_check::check(self, project) }
}

#[rustfmt::skip]
impl DomainImport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        domain_import_register::register(checkers, config)
    }
}
