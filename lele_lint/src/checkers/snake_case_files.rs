use crate::Checker;
use crate::Config;
use crate::Diagnostic;
use crate::Project;

use super::snake_case_files_check;
use super::snake_case_files_register;

pub struct SnakeCaseFiles;

impl SnakeCaseFiles {
    pub const NAME: &'static str = "snake_case_files";
    pub const CODE: &'static str = "E002";
}

#[rustfmt::skip]
impl Checker for SnakeCaseFiles {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &Project) -> Vec<Diagnostic> { snake_case_files_check::check(self, project) }
}

#[rustfmt::skip]
impl SnakeCaseFiles {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        snake_case_files_register::register(checkers, config)
    }
}

// no test_usage necessary
