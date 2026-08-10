use crate::freenet;

pub fn parse_role() -> freenet::FreenetRole {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--role" {
            match args.next().as_deref() {
                Some("subscribe") => return freenet::FreenetRole::Subscribe,
                Some("publish") => return freenet::FreenetRole::Publish,
                _ => {}
            }
        }
    }
    freenet::FreenetRole::default()
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
