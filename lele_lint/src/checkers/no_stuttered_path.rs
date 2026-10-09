use crate::Category;
use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct NoStutteredPath;

impl NoStutteredPath {
    pub const NAME: &'static str = "no_stuttered_path";
    pub const CODE: &'static str = "E025";
    pub const DOC: RuleDoc = RuleDoc {
        category: Category::Imports,
        summary: "No `meters::Meters` paths for crate-root modules: import the type once and write `Meters`.",
        why: "The module and type say the same word twice; the type name alone is already unambiguous.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod meters;
pub use meters::Meters;
mod race_length;
pub use race_length::race_length;
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
            ExampleFile {
                path: "src/race_length.rs",
                source: r"use crate::meters;

pub fn race_length() -> meters::Meters {
    meters::Meters(400)
}

#[cfg(test)]
mod tests {
    use super::race_length;

    #[test]
    fn test_usage() {
        assert_eq!(*race_length(), 400);
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
mod race_length;
pub use race_length::race_length;
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
            ExampleFile {
                path: "src/race_length.rs",
                source: r"use crate::Meters;

pub fn race_length() -> Meters {
    Meters(400)
}

#[cfg(test)]
mod tests {
    use super::race_length;

    #[test]
    fn test_usage() {
        assert_eq!(*race_length(), 400);
    }
}
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for NoStutteredPath {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(NoStutteredPath)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoStutteredPath {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoStutteredPath));
    }
}

// no test_usage necessary
