use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("not a git checkout (no .git in {0})")]
    NoGit(String),
    #[error(
        "git hook script missing at {0}: enter any crate `devenv shell` once, then re-run resync"
    )]
    HookMissing(String),
    #[error(
        "crate hook file missing: {0}/.pre-commit-config.yaml: run `cd {0} && devenv shell </dev/null` first, then re-run resync"
    )]
    YamlMissing(String),
    #[error("cannot use {path}: {message}")]
    Unusable { path: String, message: String },
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn test_usage() {
        let err = Error::YamlMissing("boxes".to_owned());
        assert!(format!("{err}").contains("boxes/.pre-commit-config.yaml"));
        let err = Error::Unusable {
            path: "p".to_owned(),
            message: "m".to_owned(),
        };
        assert_eq!(format!("{err}"), "cannot use p: m");
    }
}
