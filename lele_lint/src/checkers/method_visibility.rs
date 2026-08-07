use crate::checker::Checker;
use crate::config::Config;

use super::method_visibility_check;
use super::method_visibility_register;

pub struct MethodVisibility;

impl MethodVisibility {
    pub const NAME: &'static str = "method_visibility";
    pub const CODE: &'static str = "E003";
}

#[rustfmt::skip]
impl Checker for MethodVisibility {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { method_visibility_check::check(self, project) }
}

#[rustfmt::skip]
impl MethodVisibility {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        method_visibility_register::register(checkers, config)
    }
}

// no test_usage necessary
