use super::mode::Mode;

pub fn parse_mode() -> Mode {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--mode" {
            match args.next().as_deref() {
                Some("cli") => return Mode::Cli,
                Some("gui") => return Mode::Gui,
                _ => {}
            }
        }
    }
    Mode::default()
}

#[cfg(test)]
mod tests {
    use super::parse_mode;

    #[test]
    fn test_usage() {
        let m = parse_mode();
        let _ = m;
    }
}
