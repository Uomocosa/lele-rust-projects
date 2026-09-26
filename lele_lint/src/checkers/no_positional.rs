use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct NoPositional;

impl NoPositional {
    pub const NAME: &'static str = "no_positional";
    pub const CODE: &'static str = "E009";
}

#[rustfmt::skip]
impl Checker for NoPositional {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(NoPositional)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoPositional {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoPositional));
    }
}

// no test_usage necessary
