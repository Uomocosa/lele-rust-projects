use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::methods_layout_check;
use super::methods_layout_register;

pub struct MethodsLayout;

impl MethodsLayout {
    pub const NAME: &'static str = "methods_layout";
    pub const CODE: &'static str = "E030";
}

#[rustfmt::skip]
impl Checker for MethodsLayout {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { methods_layout_check::check(self, project) }
}

#[rustfmt::skip]
impl MethodsLayout {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        methods_layout_register::register(checkers, config)
    }
}

// no test_usage necessary
