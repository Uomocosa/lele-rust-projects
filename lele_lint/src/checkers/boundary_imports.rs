use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct BoundaryImports;

impl BoundaryImports {
    pub const NAME: &'static str = "boundary_imports";
    pub const CODE: &'static str = "E036";
    pub const DOC: RuleDoc = RuleDoc {
        category: "imports",
        summary: "A `[[lele.boundary]]` folder may not use any path from its `cannot_use` list.",
        why: "A boundary marks a pure core; forbidden imports are how I/O and frameworks leak in.",
        bad: &[
            ExampleFile {
                path: "lele.toml",
                source: r#"[[lele.boundary]]
name = "session is pure"
why = "no io"
folders = ["src/session"]
cannot_use = ["std::time"]
require = "honest"
"#,
            },
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod session;
",
            },
            ExampleFile {
                path: "src/session/mod.rs",
                source: r"mod expire;
pub use expire::expire;
",
            },
            ExampleFile {
                path: "src/session/expire.rs",
                source: r"pub fn expire(now: u64) -> u64 {
    let current = std::time::SystemTime::now();
    let _ = current;
    now.saturating_sub(1)
}
",
            },
        ],
        good: &[
            ExampleFile {
                path: "lele.toml",
                source: r#"[[lele.boundary]]
name = "session is pure"
why = "no io"
folders = ["src/session"]
cannot_use = ["std::time"]
require = "honest"
"#,
            },
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod session;
",
            },
            ExampleFile {
                path: "src/session/mod.rs",
                source: r"mod expire;
pub use expire::expire;
",
            },
            ExampleFile {
                path: "src/session/expire.rs",
                source: r"pub fn expire(now: u64) -> u64 {
    now.saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::expire;

    #[test]
    fn test_usage() {
        assert_eq!(expire(3), 2);
    }
}
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for BoundaryImports {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(BoundaryImports)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl BoundaryImports {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(BoundaryImports));
    }
}

// no test_usage necessary
