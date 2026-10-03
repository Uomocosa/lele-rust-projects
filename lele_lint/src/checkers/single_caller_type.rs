use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct SingleCallerType;

impl SingleCallerType {
    pub const NAME: &'static str = "single_caller_type";
    pub const CODE: &'static str = "E016";
    pub const DOC: RuleDoc = RuleDoc {
        category: "types",
        summary: "A type with no methods that is used from exactly one file is defined privately in that file.",
        why: "A separate file for a type only one function uses spreads one idea over two places.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod card;
",
            },
            ExampleFile {
                path: "src/card/mod.rs",
                source: r"mod card_lines;
mod render;

pub use render::render;
",
            },
            ExampleFile {
                path: "src/card/card_lines.rs",
                source: r#"#[derive(thiserror::Error, Debug)]
pub enum CardLines {
    #[error("empty card")]
    Empty,
}
"#,
            },
            ExampleFile {
                path: "src/card/render.rs",
                source: r##"use crate::card;

pub fn render(title: &str) -> Result<String, card::card_lines::CardLines> {
    if title.is_empty() {
        return Err(card::card_lines::CardLines::Empty);
    }
    Ok(format!("# {title}"))
}
"##,
            },
        ],
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod card;
",
            },
            ExampleFile {
                path: "src/card/mod.rs",
                source: r"mod render;

pub use render::render;
",
            },
            ExampleFile {
                path: "src/card/render.rs",
                source: r##"struct Lines {
    title: String,
    body: String,
}

pub fn render(title: &str, body: &str) -> String {
    let lines = Lines {
        title: title.to_string(),
        body: body.to_string(),
    };
    format!("# {}\n{}", lines.title, lines.body)
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn test_usage() {
        assert_eq!(render("A", "b"), "# A\nb");
    }
}
"##,
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for SingleCallerType {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(SingleCallerType)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl SingleCallerType {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(SingleCallerType));
    }
}

// no test_usage necessary
