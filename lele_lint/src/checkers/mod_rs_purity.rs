use crate::checkers;
use crate::Checker;
use crate::Diagnostic;
use crate::Project;
use atomic_delegate_macros::atomic_delegates;

pub struct ModRsPurity;

impl ModRsPurity {
    pub const NAME: &'static str = "mod_rs_purity";
    pub const CODE: &'static str = "E019";
}

#[rustfmt::skip]
impl Checker for ModRsPurity {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::mod_rs_purity_check::check(self, project) }
}

#[atomic_delegates]
impl ModRsPurity {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {}
}

// no test_usage necessary
