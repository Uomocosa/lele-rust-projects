use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct SingleFieldNewtype;

impl SingleFieldNewtype {
    pub const NAME: &'static str = "single_field_newtype";
    pub const CODE: &'static str = "E018";
}

#[rustfmt::skip]
impl Checker for SingleFieldNewtype {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::single_field_newtype_check::check(self, project) }
}

#[atomic_delegates]
impl SingleFieldNewtype {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
