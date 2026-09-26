use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct SingleFieldNewtype;

impl SingleFieldNewtype {
    pub const NAME: &'static str = "single_field_newtype";
    pub const CODE: &'static str = "E018";
}

#[rustfmt::skip]
impl Checker for SingleFieldNewtype {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(SingleFieldNewtype)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl SingleFieldNewtype {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(SingleFieldNewtype));
    }
}

// no test_usage necessary
