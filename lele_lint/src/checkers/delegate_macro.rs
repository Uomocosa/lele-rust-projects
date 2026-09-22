use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::delegate_macro_check;
use super::delegate_macro_register;

pub struct DelegateMacro;

impl DelegateMacro {
    pub const NAME: &'static str = "delegate_macro";
    pub const CODE: &'static str = "E032";
}

#[rustfmt::skip]
impl Checker for DelegateMacro {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { delegate_macro_check::check(self, project) }
}

#[rustfmt::skip]
impl DelegateMacro {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        delegate_macro_register::register(checkers, config)
    }
}

// no test_usage necessary
