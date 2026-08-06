use crate::checker::Checker;
use crate::config::Config;

use super::snake_case_files_check;
use super::snake_case_files_code;
use super::snake_case_files_name;
use super::snake_case_files_register;

pub struct SnakeCaseFiles;

#[rustfmt::skip]
impl Checker for SnakeCaseFiles {
    fn name(&self) -> &'static str { snake_case_files_name::name(self) }
    fn code(&self) -> &'static str { snake_case_files_code::code(self) }
    fn check(&self, project: &crate::project::Project) -> Vec<crate::diagnostic::Diagnostic> { snake_case_files_check::check(self, project) }
}

#[rustfmt::skip]
impl SnakeCaseFiles {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        snake_case_files_register::register(checkers, config)
    }
}

// no test_usage necessary
