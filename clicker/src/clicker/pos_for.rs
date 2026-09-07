use bevy::prelude::Vec2;

use crate::clicker;

#[must_use]
pub fn pos_for(index: usize, total: usize) -> Vec2 {
    let total = total.max(1);
    let cols = clicker::GRID_COLS.min(total).max(1);
    let col = index.checked_rem(cols).unwrap_or(0);
    let row = index.checked_div(cols).unwrap_or(0);
    let rows = total.div_ceil(cols);
    let half_col = f32::from(u16::try_from(cols.saturating_sub(1)).unwrap_or(0)) / 2.0;
    let half_row = f32::from(u16::try_from(rows.saturating_sub(1)).unwrap_or(0)) / 2.0;
    let x = f32::from(u16::try_from(col).unwrap_or(0)) - half_col;
    let y = half_row - f32::from(u16::try_from(row).unwrap_or(0));
    Vec2::new(x * clicker::TARGET_SPACING, y * clicker::TARGET_SPACING)
}

#[cfg(test)]
mod tests {
    use super::pos_for;
    use bevy::prelude::Vec2;

    #[test]
    fn test_usage() {
        assert_eq!(pos_for(0, 1), Vec2::ZERO);
        let a = pos_for(0, 64);
        let b = pos_for(63, 64);
        assert_ne!(a, b);
        for i in 0..64 {
            let p = pos_for(i, 64);
            assert!(p.x.abs() <= 400.0);
            assert!(p.y.abs() <= 400.0);
        }
    }
}
