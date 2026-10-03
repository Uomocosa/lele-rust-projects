use crate::Checker;
use crate::Diagnostic;
use crate::ExampleFile;
use crate::Project;
use crate::RuleDoc;
use atomic_delegate_macros::atomic_delegate;

pub struct NoStutteredType;

impl NoStutteredType {
    pub const NAME: &'static str = "no_stuttered_type";
    pub const CODE: &'static str = "E027";
    pub const DOC: RuleDoc = RuleDoc {
        category: "types",
        summary: "A type does not repeat its folder's name (`freenet::Client`, not `freenet::FreenetClient`).",
        why: "Code always says `freenet::Client` through the domain prefix, so the prefix in the type name is said twice.",
        bad: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod freenet;
",
            },
            ExampleFile {
                path: "src/freenet/mod.rs",
                source: r"mod freenet_client;

pub use freenet_client::FreenetClient;
",
            },
            ExampleFile {
                path: "src/freenet/freenet_client.rs",
                source: r#"pub struct FreenetClient {
    pub host: String,
    pub port: u16,
}

#[rustfmt::skip]
impl FreenetClient {
    pub fn url(&self) -> String { format!("ws://{}:{}", self.host, self.port) }
}

#[cfg(test)]
mod tests {
    use crate::freenet;

    #[test]
    fn test_usage() {
        let client = freenet::FreenetClient {
            host: "localhost".to_string(),
            port: 7509,
        };
        assert_eq!(client.url(), "ws://localhost:7509");
    }
}
"#,
            },
        ],
        good: &[
            ExampleFile {
                path: "src/lib.rs",
                source: r"pub mod freenet;
",
            },
            ExampleFile {
                path: "src/freenet/mod.rs",
                source: r"mod client;

pub use client::Client;
",
            },
            ExampleFile {
                path: "src/freenet/client.rs",
                source: r#"pub struct Client {
    pub host: String,
    pub port: u16,
}

#[rustfmt::skip]
impl Client {
    pub fn url(&self) -> String { format!("ws://{}:{}", self.host, self.port) }
}

#[cfg(test)]
mod tests {
    use crate::freenet;

    #[test]
    fn test_usage() {
        let client = freenet::Client {
            host: "localhost".to_string(),
            port: 7509,
        };
        assert_eq!(client.url(), "ws://localhost:7509");
    }
}
"#,
            },
        ],
    };
}

#[rustfmt::skip]
impl Checker for NoStutteredType {
    fn name(&self) -> &'static str { Self::NAME }
    fn code(&self) -> &'static str { Self::CODE }
    fn doc(&self) -> RuleDoc { Self::DOC }
    #[atomic_delegate(NoStutteredType)]
    fn check(&self, project: &Project) -> Vec<Diagnostic> {}
}

#[rustfmt::skip]
impl NoStutteredType {
    pub fn register(checkers: &mut Vec<Box<dyn Checker>>) {
        checkers.push(Box::new(NoStutteredType));
    }
}

// no test_usage necessary
