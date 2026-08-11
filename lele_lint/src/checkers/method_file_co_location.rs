use crate::checker;
use crate::config;
use crate::diagnostic;
use crate::project;

use super::method_file_co_location_check;
use super::method_file_co_location_register;

pub struct MethodFileCoLocation;

impl MethodFileCoLocation {
    pub const NAME: &'static str = "method_file_co_location";
    pub const CODE: &'static str = "E017";
}

#[rustfmt::skip]
impl checker::Checker for MethodFileCoLocation {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &project::Project) -> Vec<diagnostic::Diagnostic> { method_file_co_location_check::check(self, project) }
}

#[rustfmt::skip]
impl MethodFileCoLocation {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        method_file_co_location_register::register(checkers, config)
    }
}

// no test_usage necessary
