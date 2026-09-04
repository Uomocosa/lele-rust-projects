use crate::discovery;

#[must_use]
pub fn last_next(d: &discovery::Discovery) -> u8 {
    d.chain.values().last().map_or(0, |e| e.next)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(last_next);
    }
}
