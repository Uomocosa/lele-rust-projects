use super::enemy::Enemy;

pub fn spawn() -> Enemy {
    Enemy::default()
}

#[cfg(test)]
mod tests {
    use super::spawn;

    #[test]
    fn test_usage() {
        let e = spawn();
        assert_eq!(e.health, 10);
    }
}
