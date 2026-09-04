pub struct Creds {
    pub token: String,
    pub chat_id: String,
}

#[cfg(test)]
mod tests {
    use super::Creds;

    #[test]
    fn test_usage() {
        let c = Creds {
            token: "t".to_string(),
            chat_id: "c".to_string(),
        };
        assert_eq!(c.token, "t");
    }
}
