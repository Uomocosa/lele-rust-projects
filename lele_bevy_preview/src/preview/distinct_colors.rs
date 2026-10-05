use std::collections::HashSet;

#[must_use]
pub fn distinct_colors(pixels: &[u8]) -> usize {
    let mut seen: HashSet<u32> = HashSet::new();
    for pixel in pixels.as_chunks::<4>().0 {
        seen.insert(u32::from_be_bytes(*pixel));
    }
    seen.len()
}

#[cfg(test)]
mod tests {
    use super::distinct_colors;

    #[test]
    fn test_usage() {
        assert_eq!(distinct_colors(&[]), 0);
        assert_eq!(distinct_colors(&[0, 0, 0, 255]), 1);
        assert_eq!(
            distinct_colors(&[0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255]),
            1
        );
        assert_eq!(
            distinct_colors(&[0, 0, 0, 255, 9, 9, 9, 255, 9, 9, 9, 255, 1, 2, 3, 255]),
            3
        );
    }
}
