use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct RootReexport;

impl RootReexport {
    pub const NAME: &'static str = "root_reexport";
    pub const CODE: &'static str = "E024";
    pub const DOC: RuleDoc = RuleDoc {
        category: "imports",
        summary: "A public type in a crate-root file is re-exported from `lib.rs` (`pub use meters::Meters;`); a root fn file is a private `mod` plus `pub use`.",
        why: "Root items are used as `crate::Meters`, never `crate::meters::Meters`.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod meters;
",
            },
            ExampleFile {
                path: "src/meters.rs",
                source: r"use derive_more::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct Meters(pub u32);

#[rustfmt::skip]
impl Meters {
    pub fn doubled(&self) -> u32 { **self * 2 }
}

#[cfg(test)]
mod tests {
    use crate::meters::Meters;

    #[test]
    fn test_usage() {
        assert_eq!(Meters(4).doubled(), 8);
    }
}
",
            },
        ],
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod meters;
pub use meters::Meters;
",
            },
            ExampleFile {
                path: "src/meters.rs",
                source: r"use derive_more::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct Meters(pub u32);

#[rustfmt::skip]
impl Meters {
    pub fn doubled(&self) -> u32 { **self * 2 }
}

#[cfg(test)]
mod tests {
    use crate::Meters;

    #[test]
    fn test_usage() {
        assert_eq!(Meters(4).doubled(), 8);
    }
}
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for RootReexport {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(RootReexport)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl RootReexport {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(RootReexport));
    }
}

// no test_usage necessary
