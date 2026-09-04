#[must_use]
pub fn is_contiguous(seqs: &[u64]) -> bool {
    if seqs.is_empty() {
        return true;
    }
    let mut sorted = seqs.to_vec();
    sorted.sort_unstable();
    for (i, v) in sorted.iter().enumerate() {
        let expected = u64::try_from(i).unwrap_or(u64::MAX);
        if *v != expected {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::is_contiguous;

    #[test]
    fn test_usage() {
        assert!(is_contiguous(&[]));
        assert!(is_contiguous(&[0, 1, 2]));
        assert!(!is_contiguous(&[0, 2]));
    }
}
