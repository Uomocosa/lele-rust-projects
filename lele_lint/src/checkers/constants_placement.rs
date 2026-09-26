use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct ConstantsPlacement;

impl ConstantsPlacement {
    pub const NAME: &'static str = "constants_placement";
    pub const CODE: &'static str = "E026";
}

#[rustfmt::skip]
impl Checker for ConstantsPlacement {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(ConstantsPlacement)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl ConstantsPlacement {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(ConstantsPlacement));
    }
}

// no test_usage necessary
