use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::no_cross_domain_reexport_check;
use super::no_cross_domain_reexport_register;

pub struct NoCrossDomainReexport;

impl NoCrossDomainReexport {
    pub const NAME: &'static str = "no_cross_domain_reexport";
    pub const CODE: &'static str = "E004";
}

#[rustfmt::skip]
impl Checker for NoCrossDomainReexport {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { no_cross_domain_reexport_check::check(self, project) }
}

#[rustfmt::skip]
impl NoCrossDomainReexport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        no_cross_domain_reexport_register::register(checkers, config)
    }
}

// no test_usage necessary
