// no test_usage necessary
use crate::diagnostic::Diagnostic;
use crate::project::Project;

pub trait Checker {
    fn name(&self) -> &'static str;
    fn code(&self) -> &'static str;
    fn check(&self, project: &Project) -> Vec<Diagnostic>;
}
