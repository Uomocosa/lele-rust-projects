use lele_lint::checker::Checker;
use lele_lint::config::Config;

use super::bevy_folder_check;
use super::bevy_folder_register;

pub struct BevyFolder;

impl BevyFolder {
    pub const NAME: &'static str = "bevy_folder";
    pub const CODE: &'static str = "E008";
}

#[rustfmt::skip]
impl Checker for BevyFolder {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &lele_lint::project::Project) -> Vec<lele_lint::diagnostic::Diagnostic> { bevy_folder_check::check(self, project) }
}

#[rustfmt::skip]
impl BevyFolder {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        bevy_folder_register::register(checkers, config)
    }
}

// no test_usage necessary
