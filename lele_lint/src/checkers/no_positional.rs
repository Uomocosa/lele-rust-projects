use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct NoPositional;

impl NoPositional {
    pub const NAME: &'static str = "no_positional";
    pub const CODE: &'static str = "E009";
    pub const DOC: RuleDoc = RuleDoc {
        category: "types",
        summary: "No positional field access (`.0`, `.1`).",
        why: "A single-field newtype derefs to its value and multi-field types have names, so a number never stands for a meaning.",
        bad: &[
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
    pub fn doubled(&self) -> u32 { self.0 }
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
impl Checker for NoPositional {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(NoPositional)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoPositional {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoPositional));
    }
}

// no test_usage necessary
