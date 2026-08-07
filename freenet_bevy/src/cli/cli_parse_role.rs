use crate::freenet;

pub fn parse_role() -> (freenet::FreenetRole, bool) {
    let mut args = std::env::args().skip(1);
    let mut has_role = false;
    while let Some(arg) = args.next() {
        if arg == "--role" {
            has_role = true;
            match args.next().as_deref() {
                Some("subscribe") => return (freenet::FreenetRole::Subscribe, has_role),
                Some("publish") => return (freenet::FreenetRole::Publish, has_role),
                _ => {}
            }
        }
    }
    (freenet::FreenetRole::Publish, has_role)
}

#[cfg(test)]
mod tests {
    use super::parse_role;

    #[test]
    fn test_usage() {
        let (r, has) = parse_role();
        let _ = r;
        let _ = has;
    }
}
