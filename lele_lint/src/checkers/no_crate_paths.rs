use crate::checker;
use crate::config;
use crate::diagnostic;
use crate::project;

use super::no_crate_paths_check;
use super::no_crate_paths_register;

pub struct NoCratePaths;

impl NoCratePaths {
    pub const NAME: &'static str = "no_crate_paths";
    pub const CODE: &'static str = "E020";
}

#[rustfmt::skip]
impl checker::Checker for NoCratePaths {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &project::Project) -> Vec<diagnostic::Diagnostic> { no_crate_paths_check::check(self, project) }
}

#[rustfmt::skip]
impl NoCratePaths {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        no_crate_paths_register::register(checkers, config)
    }
}

// no test_usage necessary
