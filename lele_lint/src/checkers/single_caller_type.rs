use crate::checker::Checker;
use crate::config::Config;

use super::single_caller_type_check;
use super::single_caller_type_code;
use super::single_caller_type_name;
use super::single_caller_type_register;

pub struct SingleCallerType;

#[rustfmt::skip]
impl Checker for SingleCallerType {
    fn name(&self) -> &'static str { single_caller_type_name::name(self) }
    fn code(&self) -> &'static str { single_caller_type_code::code(self) }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { single_caller_type_check::check(self, project) }
}

#[rustfmt::skip]
impl SingleCallerType {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        single_caller_type_register::register(checkers, config)
    }
}

// no test_usage necessary
