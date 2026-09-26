use crate::checkers;
use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

pub struct NoTrivialAccessors;

impl NoTrivialAccessors {
    pub const NAME: &'static str = "no_trivial_accessors";
    pub const CODE: &'static str = "E010";
}

#[rustfmt::skip]
impl Checker for NoTrivialAccessors {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::no_trivial_accessors_check::check(self, project) }
}

#[rustfmt::skip]
impl NoTrivialAccessors {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        checkers::no_trivial_accessors_register::register(checkers, config)
    }
}

// no test_usage necessary
