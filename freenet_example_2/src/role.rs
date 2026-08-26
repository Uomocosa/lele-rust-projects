pub enum Role {
    Publish,
    Subscribe,
}

#[cfg(test)]
mod tests {
    use super::Role;

    #[test]
    fn test_usage() {
        match Role::Publish {
            Role::Publish => {}
            Role::Subscribe => panic!("expected Publish"),
        }
        match Role::Subscribe {
            Role::Subscribe => {}
            Role::Publish => panic!("expected Subscribe"),
        }
    }
}
