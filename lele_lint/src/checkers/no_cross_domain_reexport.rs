use crate::checker::Checker;
use crate::config::Config;

use super::no_cross_domain_reexport_check;
use super::no_cross_domain_reexport_code;
use super::no_cross_domain_reexport_name;
use super::no_cross_domain_reexport_register;

pub struct NoCrossDomainReexport;

#[rustfmt::skip]
impl Checker for NoCrossDomainReexport {
    fn name(&self) -> &'static str { no_cross_domain_reexport_name::name(self) }
    fn code(&self) -> &'static str { no_cross_domain_reexport_code::code(self) }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { no_cross_domain_reexport_check::check(self, project) }
}

#[rustfmt::skip]
impl NoCrossDomainReexport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        no_cross_domain_reexport_register::register(checkers, config)
    }
}

// no test_usage necessary
