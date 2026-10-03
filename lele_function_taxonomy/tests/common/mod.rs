use std::path::{Path, PathBuf};
use std::process::Command;

pub struct FixtureRun {
    pub name: String,
    pub stdout: String,
    pub stderr: String,
    pub status: Option<i32>,
}

impl FixtureRun {
    pub fn combined(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.stdout);
        if !self.stderr.is_empty() {
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&self.stderr);
        }
        out
    }
}

pub fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn fixture_dirs() -> Vec<PathBuf> {
    let root = fixtures_root();
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("lele.toml").is_file() && p.join("Cargo.toml").is_file())
        .collect();
    dirs.sort();
    dirs
}

pub fn run_fixture(dir: &Path) -> FixtureRun {
    let manifest = dir.join("Cargo.toml");
    let bin = env!("CARGO_BIN_EXE_lele-function-taxonomy");
    let name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    match Command::new(bin)
        .arg("--manifest-path")
        .arg(&manifest)
        .env(
            "LELE_TAXONOMY_DRIVER",
            env!("CARGO_BIN_EXE_lele-taxonomy-driver"),
        )
        .env_remove("LELE_BLESS")
        .output()
    {
        Ok(output) => FixtureRun {
            name,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            status: output.status.code(),
        },
        Err(err) => FixtureRun {
            name,
            stdout: String::new(),
            stderr: format!("failed to spawn lele-function-taxonomy: {err}"),
            status: None,
        },
    }
}

fn is_cargo_noise(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("Checking ")
        || t.starts_with("Compiling ")
        || t.starts_with("Finished ")
        || t.starts_with("warning: build failed")
        || t.starts_with("error: could not compile")
}

pub fn normalized_output(run: &FixtureRun) -> String {
    let mut lines: Vec<String> = run
        .combined()
        .lines()
        .map(|l| l.trim_end().replace('\\', "/"))
        .filter(|l| !l.is_empty())
        .filter(|l| !is_cargo_noise(l))
        .collect();
    lines.sort();
    let mut out = lines.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

pub fn assert_or_bless(run: &FixtureRun) {
    let expected_path = fixtures_root().join(&run.name).join("expected.txt");
    let actual = normalized_output(run);
    let trivial = actual.is_empty() || actual.contains("nothing to check");
    let expected_code = i32::from(!trivial);
    assert_eq!(
        run.status,
        Some(expected_code),
        "fixture {} exit code mismatch\noutput:\n{}",
        run.name,
        run.combined()
    );
    if std::env::var("LELE_BLESS").is_ok() {
        let written = std::fs::write(&expected_path, &actual);
        assert!(
            written.is_ok(),
            "cannot write {}: {:?}",
            expected_path.display(),
            written.err()
        );
        return;
    }
    let expected = match std::fs::read_to_string(&expected_path) {
        Ok(text) => text,
        Err(err) => {
            assert_eq!(
                err.kind(),
                std::io::ErrorKind::NotFound,
                "cannot read {}",
                expected_path.display()
            );
            String::new()
        }
    };
    assert_eq!(
        actual, expected,
        "\nfixture {} output mismatch\n--- actual ---\n{actual}--- expected ---\n{expected}",
        run.name
    );
}

// no test_usage necessary
