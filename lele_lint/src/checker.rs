use crate::Diagnostic;
use crate::Project;
use crate::RuleDoc;

pub trait Checker {
    fn name(&self) -> &'static str;
    fn code(&self) -> &'static str;
    fn doc(&self) -> RuleDoc;
    fn check(&self, project: &Project) -> Vec<Diagnostic>;
}

// no test_usage necessary
