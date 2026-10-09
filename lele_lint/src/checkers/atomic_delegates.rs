use crate::Category;
use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct AtomicDelegates;

impl AtomicDelegates {
    pub const NAME: &'static str = "atomic_delegates";
    pub const CODE: &'static str = "E012";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Delegates,
        summary: "Hand-written methods have at most 3 statements, and an impl block with one-line methods carries `#[rustfmt::skip]`.",
        why: "Longer methods belong in `methods/<type>/<method>.rs`; short ones stay readable as one line each.",
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

impl Player {
    pub fn is_alive(&self) -> bool {
        self.health > 0
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
impl Checker for AtomicDelegates {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(AtomicDelegates)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl AtomicDelegates {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(AtomicDelegates));
    }
}

// no test_usage necessary
