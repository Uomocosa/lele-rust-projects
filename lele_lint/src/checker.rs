use crate::diagnostic;
use crate::project;

pub trait Checker {
    fn name(&self) -> &'static str;
    fn code(&self) -> &'static str;
    fn check(&self, project: &project::Project) -> Vec<diagnostic::Diagnostic>;
}

// no test_usage necessary
