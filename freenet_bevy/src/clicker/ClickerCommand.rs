pub enum ClickerCommand {
    Increment { count: u64 },
}

#[cfg(test)]
mod tests {
    use super::ClickerCommand;

    #[test]
    fn test_usage() {
        let cmd = ClickerCommand::Increment { count: 42 };
        match cmd {
            ClickerCommand::Increment { count } => assert_eq!(count, 42),
        }
    }
}
