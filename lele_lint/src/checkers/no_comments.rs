use crate::checkers;
use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

pub struct NoComments;

impl NoComments {
    pub const NAME: &'static str = "no_comments";
    pub const CODE: &'static str = "E031";
}

#[rustfmt::skip]
impl Checker for NoComments {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { checkers::no_comments_check::check(self, project) }
}

#[rustfmt::skip]
impl NoComments {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        checkers::no_comments_register::register(checkers, config)
    }
}

// no test_usage necessary
