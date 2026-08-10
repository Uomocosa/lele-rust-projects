use crate::checker;
use crate::config;

use super::single_field_newtype_check;
use super::single_field_newtype_register;

pub struct SingleFieldNewtype;

impl SingleFieldNewtype {
    pub const NAME: &'static str = "single_field_newtype";
    pub const CODE: &'static str = "E018";
}

#[rustfmt::skip]
impl checker::Checker for SingleFieldNewtype {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { single_field_newtype_check::check(self, project) }
}

#[rustfmt::skip]
impl SingleFieldNewtype {
    pub fn register(checkers: &mut Vec<Box<dyn checker::Checker>>, config: &config::Config) {
        single_field_newtype_register::register(checkers, config)
    }
}

// no test_usage necessary
