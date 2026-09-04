use crate::discovery;

#[must_use]
pub fn next_seq(d: &discovery::Discovery) -> u64 {
    d.chain
        .keys()
        .copied()
        .max()
        .map_or(0, |m| m.checked_add(1).unwrap_or(m))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(next_seq);
    }
}
