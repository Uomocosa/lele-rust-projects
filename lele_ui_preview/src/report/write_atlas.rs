use std::path::Path;

use image::imageops::FilterType;
use image::{Rgba, RgbaImage};

use crate::Error;
use crate::report;

pub fn write_atlas(
    out_dir: &Path,
    group: &str,
    screen: &str,
    pngs: &[String],
) -> Result<String, Error> {
    let mut thumbs = Vec::new();
    for png in pngs {
        let bytes = std::fs::read(out_dir.join(png))?;
        let img =
            image::load_from_memory(&bytes).map_err(|e| Error::Image(format!("{png}: {e}")))?;
        let height = scaled_height(img.width(), img.height());
        thumbs.push(
            img.resize_exact(report::ATLAS_CELL_WIDTH, height, FilterType::Triangle)
                .to_rgba8(),
        );
    }
    let canvas = compose(&thumbs);
    let relative = format!(
        "{}/{}/{}",
        report::slug(group),
        report::slug(screen),
        report::ATLAS_FILE
    );
    let path = out_dir.join(&relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    canvas
        .save(&path)
        .map_err(|e| Error::Image(format!("{}: {e}", path.display())))?;
    Ok(relative)
}

// needed helper: keep the aspect ratio at a fixed thumbnail width
fn scaled_height(width: u32, height: u32) -> u32 {
    let scaled = u64::from(height)
        .saturating_mul(u64::from(report::ATLAS_CELL_WIDTH))
        .checked_div(u64::from(width))
        .unwrap_or(1)
        .max(1);
    u32::try_from(scaled).unwrap_or(u32::MAX)
}

// needed helper: lay thumbnails out in a fixed-column grid
fn compose(thumbs: &[RgbaImage]) -> RgbaImage {
    let columns = usize::try_from(report::ATLAS_COLUMNS).unwrap_or(1);
    let gap = report::ATLAS_GAP;
    let cell = report::ATLAS_CELL_WIDTH;
    let used_columns = u32::try_from(thumbs.len().min(columns)).unwrap_or(1).max(1);
    let row_heights: Vec<u32> = thumbs
        .chunks(columns)
        .map(|row| row.iter().map(RgbaImage::height).max().unwrap_or(0))
        .collect();
    let width = used_columns
        .saturating_mul(cell)
        .saturating_add(used_columns.saturating_add(1).saturating_mul(gap));
    let rows = u32::try_from(row_heights.len()).unwrap_or(0);
    let height = row_heights
        .iter()
        .fold(0u32, |acc, h| acc.saturating_add(*h))
        .saturating_add(rows.saturating_add(1).saturating_mul(gap));
    let mut canvas = RgbaImage::from_pixel(width, height.max(1), Rgba([226, 228, 234, 255]));
    let mut y = gap;
    for (row, row_height) in thumbs.chunks(columns).zip(&row_heights) {
        let mut x = gap;
        for thumb in row {
            image::imageops::overlay(&mut canvas, thumb, i64::from(x), i64::from(y));
            x = x.saturating_add(cell).saturating_add(gap);
        }
        y = y.saturating_add(*row_height).saturating_add(gap);
    }
    canvas
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};

    use super::write_atlas;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().unwrap();
        let mut pngs = Vec::new();
        for i in 0..5 {
            let rel = format!("s{i}.png");
            RgbaImage::from_pixel(640, 480, Rgba([10, 20, 30, 255]))
                .save(dir.path().join(&rel))
                .unwrap();
            pngs.push(rel);
        }
        let rel = write_atlas(dir.path(), "desktop", "/", &pngs).unwrap();
        let atlas = image::open(dir.path().join(rel)).unwrap();
        assert_eq!(atlas.width(), 4 * 320 + 5 * 8);
        assert_eq!(atlas.height(), 2 * 240 + 3 * 8);
    }
}
