use crate::freenet;

pub fn parse_role() -> Option<freenet::FreenetRole> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--role" {
            match args.next().as_deref() {
                Some("subscribe") => return Some(freenet::FreenetRole::Subscribe),
                Some("publish") => return Some(freenet::FreenetRole::Publish),
                _ => {}
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_role;

    #[test]
    fn test_usage() {
        let r = parse_role();
        let _ = r;
    }
}
