use lele_lint::checker::Checker;
use lele_lint::config::Config;

use super::bevy_ui_check;
use super::bevy_ui_register;

pub struct BevyUi;

impl BevyUi {
    pub const NAME: &'static str = "bevy_ui";
    pub const CODE: &'static str = "E029";
}

#[rustfmt::skip]
impl Checker for BevyUi {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn check(&self, project: &lele_lint::project::Project) -> Vec<lele_lint::diagnostic::Diagnostic> { bevy_ui_check::check(self, project) }
}

#[rustfmt::skip]
impl BevyUi {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>, config: &Config) {
        bevy_ui_register::register(checkers, config)
    }
}

// no test_usage necessary
