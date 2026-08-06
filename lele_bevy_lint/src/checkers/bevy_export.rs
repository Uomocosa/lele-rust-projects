use lele_lint::checker::Checker;
use lele_lint::config::Config;

use super::bevy_export_check;
use super::bevy_export_code;
use super::bevy_export_name;
use super::bevy_export_register;

pub struct BevyExport;

#[rustfmt::skip]
impl Checker for BevyExport {
    fn name(&self) -> &'static str { bevy_export_name::name(self) }
    fn code(&self) -> &'static str { bevy_export_code::code(self) }
    fn check(&self, project: &lele_lint::project::Project) -> Vec<lele_lint::diagnostic::Diagnostic> { bevy_export_check::check(self, project) }
}

#[rustfmt::skip]
impl BevyExport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        bevy_export_register::register(checkers, config)
    }
}

// no test_usage necessary
