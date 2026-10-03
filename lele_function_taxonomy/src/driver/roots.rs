pub const DEFAULT_ROOTS: &[&str] = &[
    "std::time::SystemTime::now",
    "std::time::Instant::now",
    "std::fs::",
    "std::path::Path::exists",
    "std::path::Path::is_file",
    "std::path::Path::is_dir",
    "std::path::Path::metadata",
    "std::path::Path::read_dir",
    "std::path::Path::canonicalize",
    "std::net::",
    "std::env::",
    "std::process::",
    "std::io::stdin",
    "std::io::stdout",
    "std::io::stderr",
    "std::io::_print",
    "std::io::_eprint",
    "std::thread::sleep",
    "std::thread::LocalKey::with",
    "std::thread::LocalKey::try_with",
    "rand::",
    "getrandom::",
    "tokio::fs::",
    "tokio::net::",
    "tokio::time::",
    "tokio::io::",
    "tokio::process::",
    "tokio::signal::",
];

// needed helper: strip `::<...>` turbofish so `LocalKey::<T>::with` matches `LocalKey::with`
fn normalize(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut depth: u32 = 0;
    for ch in path.chars() {
        match ch {
            '<' => {
                depth = depth.saturating_add(1);
                if out.ends_with("::") {
                    out.truncate(out.len().saturating_sub(2));
                }
            }
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out
}

// needed helper: trait-impl paths print `<Self as Trait>::method`; also test the Self type
fn self_types(path: &str) -> Vec<String> {
    let Some(rest) = path.strip_prefix('<') else {
        return Vec::new();
    };
    let Some((ty, _)) = rest.split_once(" as ") else {
        return Vec::new();
    };
    vec![format!("{ty}::")]
}

fn matches_one(path: &str, root: &str) -> bool {
    if root.ends_with("::") {
        path.starts_with(root) || path.contains(root)
    } else {
        path == root || path.starts_with(&format!("{root}::"))
    }
}

// needed helper: logging is honest everywhere; tracing/log callsite statics are sanctioned
pub fn is_logging_type(type_path: &str) -> bool {
    type_path.starts_with("tracing_core::callsite::")
        || type_path.starts_with("tracing::")
        || type_path.starts_with("log::")
}

pub fn is_io_root(path: &str, extra: &[String]) -> Option<String> {
    let normalized = normalize(path);
    let mut candidates = self_types(path);
    candidates.push(normalized);

    for candidate in &candidates {
        for root in DEFAULT_ROOTS {
            if matches_one(candidate, root) {
                return Some((*root).to_string());
            }
        }
        for root in extra {
            let root_norm = normalize(root);
            if matches_one(candidate, &root_norm) {
                return Some(root.clone());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{is_io_root, normalize};

    #[test]
    fn test_usage() {
        assert_eq!(
            normalize("std::thread::LocalKey::<T>::with"),
            "std::thread::LocalKey::with"
        );
        assert!(is_io_root("std::time::SystemTime::now", &[]).is_some());
        assert!(is_io_root("std::thread::LocalKey::<T>::with", &[]).is_some());
        assert!(is_io_root("<std::fs::File as std::io::Read>::read", &[]).is_some());
        assert!(is_io_root("tokio::time::Instant::now", &[]).is_some());
        assert!(is_io_root("std::sync::atomic::Atomic::<u32>::fetch_add", &[]).is_none());
        assert!(is_io_root("my_crate::helper", &[]).is_none());
        assert!(super::is_logging_type(
            "tracing_core::callsite::DefaultCallsite"
        ));
        assert!(super::is_logging_type("tracing::foo::Bar"));
        assert!(super::is_logging_type("log::foo::Baz"));
        assert!(!super::is_logging_type("std::sync::OnceLock"));
        assert!(is_io_root(
            "my_crate::read_sensor",
            &["my_crate::read_sensor".to_string()]
        )
        .is_some());
    }
}
