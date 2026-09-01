pub enum Command {
    Increment { count: u64 },
}

#[cfg(test)]
mod tests {
    use super::Command;

    #[test]
    fn test_usage() {
        let cmd = Command::Increment { count: 42 };
        match cmd {
            Command::Increment { count } => assert_eq!(count, 42),
        }
    }
}
