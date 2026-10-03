use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct Vocabulary;

impl Vocabulary {
    pub const NAME: &'static str = "vocabulary";
    pub const CODE: &'static str = "E035";
    pub const DOC: RuleDoc = RuleDoc {
        category: "style",
        summary: "Names we declare use the crate's vocabulary; banned synonyms are reported.",
        why: "One name per concept keeps code searchable; the declaration lives in `lele.toml`.",
        bad: &[
            ExampleFile {
                path: "lele.toml",
                source: r#"[[lele.vocabulary]]
name = "directory"
meaning = "The shared list of open rooms."
banned = ["board"]
"#,
            },
            ExampleFile {
                path: "src/lib.rs",
                source: r"mod merge_board;
pub use merge_board::merge_board;
",
            },
            ExampleFile {
                path: "src/merge_board.rs",
                source: r#"pub fn merge_board(rooms: &[&str]) -> usize {
    rooms.len()
}

#[cfg(test)]
mod tests {
    use super::merge_board;

    #[test]
    fn test_usage() {
        assert_eq!(merge_board(&["a"]), 1);
    }
}
"#,
            },
        ],
        good: &[
            ExampleFile {
                path: "lele.toml",
                source: r#"[[lele.vocabulary]]
name = "directory"
meaning = "The shared list of open rooms."
banned = ["board"]
"#,
            },
            ExampleFile {
                path: "src/lib.rs",
                source: r"mod merge_directory;
pub use merge_directory::merge_directory;
",
            },
            ExampleFile {
                path: "src/merge_directory.rs",
                source: r#"pub fn merge_directory(rooms: &[&str]) -> usize {
    rooms.len()
}

#[cfg(test)]
mod tests {
    use super::merge_directory;

    #[test]
    fn test_usage() {
        assert_eq!(merge_directory(&["a"]), 1);
    }
}
"#,
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for Vocabulary {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(Vocabulary)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl Vocabulary {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(Vocabulary));
    }
}

// no test_usage necessary
