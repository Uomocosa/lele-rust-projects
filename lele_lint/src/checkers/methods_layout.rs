use crate::Category;
use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct MethodsLayout;

impl MethodsLayout {
    pub const NAME: &'static str = "methods_layout";
    pub const CODE: &'static str = "E030";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Delegates,
        summary: "Every `#[atomic_delegates]` shell has a body at `methods/<type>/<method>.rs` with `pub fn <method>`, 2+ statements and its own `test_usage`.",
        why: "`methods/` hides bodies until you need them; the type file stays a short list of what the type can do.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r#"pub mod player;

#[path = "../methods/mod.rs"]
pub mod methods;
"#,
            },
            ExampleFile {
                path: "src/player/mod.rs",
                source: r"mod player;

pub use player::Player;
",
            },
            ExampleFile {
                path: "src/player/player.rs",
                source: r"use atomic_delegate_macros::atomic_delegates;

pub struct Player {
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

#[atomic_delegates]
impl Player {
    pub fn heal(&mut self, amount: u32) {}
}

#[cfg(test)]
mod tests {
    use crate::player;

    #[test]
    fn test_usage() {
        let mut player = player::Player::default();
        player.heal(5);
        assert_eq!(player.health, 100);
    }
}
",
            },
        ],
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r#"pub mod player;

#[path = "../methods/mod.rs"]
pub mod methods;
"#,
            },
            ExampleFile {
                path: "src/player/mod.rs",
                source: r"mod player;

pub use player::Player;
",
            },
            ExampleFile {
                path: "src/player/player.rs",
                source: r"use atomic_delegate_macros::atomic_delegates;

pub struct Player {
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

#[atomic_delegates]
impl Player {
    pub fn heal(&mut self, amount: u32) {}
}

#[cfg(test)]
mod tests {
    use crate::player;

    #[test]
    fn test_usage() {
        let mut player = player::Player::default();
        player.heal(5);
        assert_eq!(player.health, 100);
    }
}
",
            },
            ExampleFile {
                path: "methods/mod.rs",
                source: r"pub mod player;
",
            },
            ExampleFile {
                path: "methods/player/mod.rs",
                source: r"mod heal;
pub use heal::heal;
",
            },
            ExampleFile {
                path: "methods/player/heal.rs",
                source: r"use crate::player;

pub fn heal(player: &mut player::Player, amount: u32) {
    let healed = player.health.saturating_add(amount);
    player.health = healed.min(100);
}

#[cfg(test)]
mod tests {
    use super::heal;
    use crate::player::Player;

    #[test]
    fn test_usage() {
        let mut player = Player {
            name: String::new(),
            health: 10,
        };
        heal(&mut player, 5);
        assert_eq!(player.health, 15);
    }
}
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for MethodsLayout {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(MethodsLayout)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl MethodsLayout {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(MethodsLayout));
    }
}

// no test_usage necessary
