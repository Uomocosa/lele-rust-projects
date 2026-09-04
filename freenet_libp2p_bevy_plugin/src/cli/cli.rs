use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg(long, default_value = "blackboard-v1")]
    pub namespace: String,
    #[arg(long)]
    pub identity_dir: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn test_usage() {
        let cli = Cli::try_parse_from(["prog"]).unwrap();
        assert_eq!(cli.namespace, "blackboard-v1");
    }
}
