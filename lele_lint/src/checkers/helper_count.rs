use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct HelperCount;

impl HelperCount {
    pub const NAME: &'static str = "helper_count";
    pub const CODE: &'static str = "E015";
    pub const DOC: RuleDoc = RuleDoc {
        category: "layout",
        summary: "A file has one public function; at most 2 private helpers unless each is marked `// needed helper: <why>`.",
        why: "Helpers pile up silently; the marker forces a reason for every extra function, and unexplained ones get their own file.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"mod score;
pub use score::score;
",
            },
            ExampleFile {
                path: "src/score.rs",
                source: r"pub fn score(hits: u32, misses: u32) -> u32 {
    bonus(hits).saturating_add(base(hits)).saturating_sub(penalty(misses))
}

fn base(hits: u32) -> u32 {
    hits.saturating_mul(10)
}

fn bonus(hits: u32) -> u32 {
    if hits > 10 { 50 } else { 0 }
}

fn penalty(misses: u32) -> u32 {
    misses.saturating_mul(5)
}

#[cfg(test)]
mod tests {
    use super::score;

    #[test]
    fn test_usage() {
        assert_eq!(score(1, 0), 10);
    }
}
",
            },
        ],
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"mod score;
pub use score::score;
",
            },
            ExampleFile {
                path: "src/score.rs",
                source: r"pub fn score(hits: u32, misses: u32) -> u32 {
    bonus(hits).saturating_add(base(hits)).saturating_sub(penalty(misses))
}

// needed helper: points per hit
fn base(hits: u32) -> u32 {
    hits.saturating_mul(10)
}

// needed helper: streak bonus above 10 hits
fn bonus(hits: u32) -> u32 {
    if hits > 10 { 50 } else { 0 }
}

// needed helper: points lost per miss
fn penalty(misses: u32) -> u32 {
    misses.saturating_mul(5)
}

#[cfg(test)]
mod tests {
    use super::score;

    #[test]
    fn test_usage() {
        assert_eq!(score(1, 0), 10);
    }
}
",
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for HelperCount {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(HelperCount)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl HelperCount {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(HelperCount));
    }
}

// no test_usage necessary
