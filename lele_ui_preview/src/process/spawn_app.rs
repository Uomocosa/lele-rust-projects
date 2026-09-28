use std::collections::BTreeMap;
use std::fs::File;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::Error;

pub fn spawn_app(
    command: &[String],
    cwd: &Path,
    env: &BTreeMap<String, String>,
    log: &Path,
) -> Result<Child, Error> {
    let Some((program, args)) = command.split_first() else {
        return Err(Error::Config("empty command".to_string()));
    };
    let out = File::create(log)?;
    let err = out.try_clone()?;
    let child = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .envs(env)
        .stdin(Stdio::null())
        .stdout(Stdio::from(out))
        .stderr(Stdio::from(err))
        .process_group(0)
        .spawn()?;
    Ok(child)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::spawn_app;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("app.log");
        let command = vec!["sh".to_string(), "-c".to_string(), "echo hi".to_string()];
        let mut child = spawn_app(&command, dir.path(), &BTreeMap::new(), &log).unwrap();
        assert!(child.wait().unwrap().success());
        assert_eq!(std::fs::read_to_string(log).unwrap().trim(), "hi");
    }
}
