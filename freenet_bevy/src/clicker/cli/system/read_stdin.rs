use std::io::{self, BufRead};

use bevy::prelude::*;

use crate::clicker::ClickerCommand;
use crate::clicker::resource::State::ClickerState;

pub fn read_stdin(mut state: ResMut<ClickerState>) {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    if let Some(Ok(line)) = lines.next() {
        let line = line.trim().to_lowercase();
        match line.as_str() {
            "increment" | "inc" | "+" => {
                state.count = state.count.wrapping_add(1);
                let cmd = ClickerCommand::Increment { count: state.count };
                let _ = state.cmd_tx.send(cmd);
                println!("> incremented to {}", state.count);
            }
            "status" | "s" => {
                println!("> current count: {}", state.count);
            }
            "quit" | "q" | "exit" => {
                println!("> quitting...");
                std::process::exit(0);
            }
            "help" | "h" => {
                println!("Commands:");
                println!("  increment, inc, +  - Increment the counter");
                println!("  status, s          - Show current count");
                println!("  quit, q, exit      - Quit the application");
                println!("  help, h            - Show this help");
            }
            _ => {
                println!(
                    "> unknown command: '{}'. Type 'help' for available commands.",
                    line
                );
            }
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
