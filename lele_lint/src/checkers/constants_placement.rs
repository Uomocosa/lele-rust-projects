use crate::Category;
use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct ConstantsPlacement;

impl ConstantsPlacement {
    pub const NAME: &'static str = "constants_placement";
    pub const CODE: &'static str = "E026";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Layout,
        summary: "A constant lives in the nearest `constants.rs` (or `__basic__/constants.rs`) shared by all its users.",
        why: "Constants placed too high leak across domains; placed too low they get duplicated.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod constants;
pub mod player;
",
            },
            ExampleFile {
                path: "src/constants.rs",
                source: r"pub const MAX_HEALTH: u32 = 100;
",
            },
            ExampleFile {
                path: "src/player/mod.rs",
                source: r"mod clamp_health;

pub use clamp_health::clamp_health;
",
            },
            ExampleFile {
                path: "src/player/clamp_health.rs",
                source: r"use crate::constants;

pub fn clamp_health(health: u32) -> u32 {
    health.min(constants::MAX_HEALTH)
}

#[cfg(test)]
mod tests {
    use super::clamp_health;

    #[test]
    fn test_usage() {
        assert_eq!(clamp_health(150), 100);
    }
}
",
            },
        ],
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod player;
",
            },
            ExampleFile {
                path: "src/player/mod.rs",
                source: r#"mod clamp_health;
#[path = "__basic__/mod.rs"]
pub mod basic;

pub use basic::constants::MAX_HEALTH;
pub use clamp_health::clamp_health;
"#,
            },
            ExampleFile {
                path: "src/player/__basic__/mod.rs",
                source: r"pub mod constants;
",
            },
            ExampleFile {
                path: "src/player/__basic__/constants.rs",
                source: r"pub const MAX_HEALTH: u32 = 100;
",
            },
            ExampleFile {
                path: "src/player/clamp_health.rs",
                source: r"use crate::player;

pub fn clamp_health(health: u32) -> u32 {
    health.min(player::MAX_HEALTH)
}

#[cfg(test)]
mod tests {
    use super::clamp_health;

    #[test]
    fn test_usage() {
        assert_eq!(clamp_health(150), 100);
    }
}
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for ConstantsPlacement {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(ConstantsPlacement)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl ConstantsPlacement {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(ConstantsPlacement));
    }
}

// no test_usage necessary
