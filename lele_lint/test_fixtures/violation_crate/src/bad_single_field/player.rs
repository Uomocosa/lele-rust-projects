pub struct Player {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::Player;

    #[test]
    fn test_usage() {
        let p = Player { name: String::new() };
        let _ = p;
    }
}
