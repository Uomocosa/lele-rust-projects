use crate::checker;
use crate::config;
use crate::diagnostic;
use crate::project;

use super::method_visibility_check;
use super::method_visibility_register;

pub struct MethodVisibility;

impl MethodVisibility {
    pub const NAME: &'static str = "method_visibility";
    pub const CODE: &'static str = "E003";
}

#[rustfmt::skip]
impl checker::Checker for MethodVisibility {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &project::Project) -> Vec<diagnostic::Diagnostic> { method_visibility_check::check(self, project) }
}

#[rustfmt::skip]
impl MethodVisibility {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        method_visibility_register::register(checkers, config)
    }
}

// no test_usage necessary
