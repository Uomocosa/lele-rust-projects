use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::single_field_newtype_check;
use super::single_field_newtype_register;

pub struct SingleFieldNewtype;

impl SingleFieldNewtype {
    pub const NAME: &'static str = "single_field_newtype";
    pub const CODE: &'static str = "E018";
}

#[rustfmt::skip]
impl Checker for SingleFieldNewtype {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { single_field_newtype_check::check(self, project) }
}

#[rustfmt::skip]
impl SingleFieldNewtype {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        single_field_newtype_register::register(checkers, config)
    }
}

// no test_usage necessary
