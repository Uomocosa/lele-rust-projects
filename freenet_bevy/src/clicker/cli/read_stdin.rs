use std::io::{self, BufRead};

use bevy::prelude::*;

use crate::clicker::cli::cli_command::CliCommand;

pub fn read_stdin(mut writer: MessageWriter<CliCommand>) {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    if let Some(Ok(line)) = lines.next() {
        match CliCommand::parse(&line) {
            Some(cmd) => {
                writer.write(cmd);
            }
            None => println!("> unknown command: '{}'", line.trim()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::read_stdin;

    #[test]
    fn test_usage() {
        let _ = read_stdin;
    }
}
