use std::collections::HashSet;

pub fn changed_lines(old: &str, new: &str) -> Vec<usize> {
    let before: HashSet<&str> = old.lines().map(str::trim).collect();
    new.lines()
        .enumerate()
        .filter(|(_, line)| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !before.contains(trimmed)
        })
        .map(|(i, _)| i.saturating_add(1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::changed_lines;

    #[test]
    fn test_usage() {
        let old = "fn a() {}\n\nfn b() {}\n";
        let new = "fn a() {}\n\nfn b() { 1 }\nfn c() {}\n";
        assert_eq!(changed_lines(old, new), vec![3, 4]);
        assert_eq!(changed_lines(old, old), Vec::<usize>::new());
    }
}
