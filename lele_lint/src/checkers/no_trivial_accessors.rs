use crate::Category;
use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct NoTrivialAccessors;

impl NoTrivialAccessors {
    pub const NAME: &'static str = "no_trivial_accessors";
    pub const CODE: &'static str = "E010";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Types,
        summary: "No getters or setters that only read or write a public field.",
        why: "The field is already public; an accessor adds a second name for the same thing.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod player;
",
            },
            ExampleFile {
                path: "src/player/mod.rs",
                source: r"mod player;

pub use player::Player;
",
            },
            ExampleFile {
                path: "src/player/player.rs",
                source: r"pub struct Player {
    pub name: String,
    pub health: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            name: String::new(),
            health: 100,
        }
    }
}

#[rustfmt::skip]
impl Player {
    pub fn health(&self) -> u32 { self.health }
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
                source: r"mod player;

pub use player::Player;
",
            },
            ExampleFile {
                path: "src/player/player.rs",
                source: r"pub struct Player {
    pub name: String,
    pub health: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            name: String::new(),
            health: 100,
        }
    }
}

#[rustfmt::skip]
impl Player {
    pub fn is_alive(&self) -> bool { self.health > 0 }
}

#[cfg(test)]
mod tests {
    use crate::player;

    #[test]
    fn test_usage() {
        assert!(player::Player::default().is_alive());
    }
}
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for NoTrivialAccessors {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(NoTrivialAccessors)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoTrivialAccessors {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoTrivialAccessors));
    }
}

// no test_usage necessary
