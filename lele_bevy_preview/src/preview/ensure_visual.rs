use std::path::Path;

use crate::Error;
use crate::preview;

pub fn ensure_visual(pixels: &[u8], out: &Path, min: usize) -> Result<(), Error> {
    let distinct = preview::distinct_colors::distinct_colors(pixels);
    if distinct < min {
        return Err(Error::NoVisual {
            path: out.display().to_string(),
            distinct,
            min,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ensure_visual;
    use crate::Error;
    use std::path::Path;

    #[test]
    fn test_usage() {
        let uniform = [3_u8, 4, 5, 255, 3, 4, 5, 255];
        let varied = [3_u8, 4, 5, 255, 9, 8, 7, 255];
        let path = Path::new("out.png");
        assert!(ensure_visual(&varied, path, 2).is_ok());
        assert!(matches!(
            ensure_visual(&uniform, path, 2),
            Err(Error::NoVisual { distinct: 1, .. })
        ));
        assert!(ensure_visual(&uniform, path, 1).is_ok());
    }
}
