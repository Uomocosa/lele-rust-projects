use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegate;

pub struct ModRsPurity;

impl ModRsPurity {
    pub const NAME: &'static str = "mod_rs_purity";
    pub const CODE: &'static str = "E019";
}

#[rustfmt::skip]
impl Checker for ModRsPurity {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    #[atomic_delegate(ModRsPurity)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl ModRsPurity {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(ModRsPurity));
    }
}

// no test_usage necessary
