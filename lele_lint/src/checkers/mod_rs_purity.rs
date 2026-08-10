use crate::checker;
use crate::config;

use super::mod_rs_purity_check;
use super::mod_rs_purity_register;

pub struct ModRsPurity;

impl ModRsPurity {
    pub const NAME: &'static str = "mod_rs_purity";
    pub const CODE: &'static str = "E019";
}

#[rustfmt::skip]
impl checker::Checker for ModRsPurity {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { mod_rs_purity_check::check(self, project) }
}

#[rustfmt::skip]
impl ModRsPurity {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        mod_rs_purity_register::register(checkers, config)
    }
}

// no test_usage necessary
