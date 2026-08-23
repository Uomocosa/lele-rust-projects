use crate::boxes;

pub fn pick_spawn_x(existing_xs: &[f32]) -> f32 {
    let bound = boxes::GROUND_WIDTH / 2.0 - boxes::BOX_SIZE / 2.0;
    let mut xs: Vec<f32> = existing_xs.to_vec();
    xs.push(-bound);
    xs.push(bound);
    xs.sort_by(f32::total_cmp);

    let mut widest_mid = 0.0;
    let mut widest_gap = 0.0f32;
    for pair in xs.windows(2) {
        let gap = pair[1] - pair[0];
        if gap > widest_gap {
            widest_gap = gap;
            widest_mid = (pair[0] + pair[1]) / 2.0;
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
