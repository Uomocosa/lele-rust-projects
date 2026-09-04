use crate::boxes;

#[must_use]
pub fn pick_spawn_x(existing_xs: &[f32]) -> f32 {
    let bound = boxes::GROUND_WIDTH / 2.0 - boxes::BOX_SIZE / 2.0;
    let mut xs: Vec<f32> = existing_xs.to_vec();
    xs.push(-bound);
    xs.push(bound);
    xs.sort_by(f32::total_cmp);

    let mut widest_mid = 0.0;
    let mut widest_gap = 0.0f32;
    for pair in xs.windows(2) {
        let Some(&a) = pair.first() else {
            continue;
        };
        let Some(&b) = pair.get(1) else {
            continue;
        };
        let gap = b - a;
        if gap > widest_gap {
            widest_gap = gap;
            widest_mid = a.midpoint(b);
        }
    }
    widest_mid
}

#[cfg(test)]
mod tests {
    use super::pick_spawn_x;
    use crate::boxes;

    #[test]
    fn test_usage() {
        let bound = boxes::GROUND_WIDTH / 2.0 - boxes::BOX_SIZE / 2.0;

        assert_eq!(pick_spawn_x(&[]), 0.0);

        let x = pick_spawn_x(&[0.0]);
        assert!((x.abs() - bound / 2.0).abs() < f32::EPSILON);

        let left = pick_spawn_x(&[0.0]);
        let right = pick_spawn_x(&[0.0, left]);
        assert_ne!(left, right);
        assert!((left + right).abs() < f32::EPSILON);

        let between = pick_spawn_x(&[-bound, bound]);
        assert_eq!(between, 0.0);
    }
}
