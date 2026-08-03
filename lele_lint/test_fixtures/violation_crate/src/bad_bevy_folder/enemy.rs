use super::enemy_spawn;

pub struct Enemy {
    pub health: u32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self { health: 10 }
    }
}

#[rustfmt::skip]
impl Enemy {
    pub fn spawn() -> Self { enemy_spawn::spawn() }
}

#[cfg(test)]
mod tests {
    use crate::bad_bevy_folder;

    #[test]
    fn test_usage() {
        let e = bad_bevy_folder::Enemy::spawn();
        assert_eq!(e.health, 10);
    }
}
