use std::collections::BTreeMap;
use std::path::Path;

use crate::index;

const MIN_GROUP_SIZE: usize = 2;

pub fn folder_groups(files: &[&Path]) -> Vec<index::ItemGroup> {
    let mut folder_of: Vec<Vec<String>> = files.iter().map(|file| domain_folder(file)).collect();
    loop {
        let counts = count_by_folder(&folder_of);
        let undersized = |folder: &Vec<String>| {
            !folder.is_empty() && counts.get(folder).copied().unwrap_or(0) < MIN_GROUP_SIZE
        };
        let Some(depth) = folder_of
            .iter()
            .filter(|f| undersized(f))
            .map(Vec::len)
            .max()
        else {
            break;
        };
        let lift: Vec<bool> = folder_of
            .iter()
            .map(|f| f.len() == depth && undersized(f))
            .collect();
        for (folder, lift) in folder_of.iter_mut().zip(lift) {
            if lift {
                folder.pop();
            }
        }
    }
    let mut members: BTreeMap<Vec<String>, Vec<usize>> = BTreeMap::new();
    for (node, folder) in folder_of.into_iter().enumerate() {
        members.entry(folder).or_default().push(node);
    }
    members
        .into_iter()
        .filter(|(_, m)| m.len() >= MIN_GROUP_SIZE)
        .map(|(folder, m)| index::ItemGroup {
            folder: folder.join("/"),
            members: m,
        })
        .collect()
}

// needed helper: parent folders of a source file, dunder containers folded into their domain
fn domain_folder(file: &Path) -> Vec<String> {
    let mut parts: Vec<String> = file
        .parent()
        .map(|dir| {
            dir.components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    if parts.first().is_some_and(|p| p == "src") {
        parts.remove(0);
    }
    while parts.last().is_some_and(|p| p.starts_with("__")) {
        parts.pop();
    }
    parts
}

// needed helper: how many nodes currently sit in each folder
fn count_by_folder(folder_of: &[Vec<String>]) -> BTreeMap<Vec<String>, usize> {
    let mut counts: BTreeMap<Vec<String>, usize> = BTreeMap::new();
    for folder in folder_of {
        let entry = counts.entry(folder.clone()).or_insert(0);
        *entry = entry.saturating_add(1);
    }
    counts
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::folder_groups;

    #[test]
    fn test_usage() {
        let files = [
            Path::new("src/p2p/run/start.rs"),
            Path::new("src/p2p/run/stop.rs"),
            Path::new("src/p2p/__basic__/structs.rs"),
            Path::new("src/p2p/codec/encode.rs"),
            Path::new("src/roster/join.rs"),
            Path::new("src/lib.rs"),
        ];
        let groups = folder_groups(&files);
        let names: Vec<&str> = groups.iter().map(|g| g.folder.as_str()).collect();
        assert_eq!(names, vec!["", "p2p", "p2p/run"]);
        let p2p = groups.iter().find(|g| g.folder == "p2p").unwrap();
        assert_eq!(p2p.members, vec![2, 3]);
        let root = groups.iter().find(|g| g.folder.is_empty()).unwrap();
        assert_eq!(root.members, vec![4, 5]);
    }
}
