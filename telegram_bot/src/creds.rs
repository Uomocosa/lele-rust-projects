pub struct Creds {
    pub token: String,
    pub chat_id: String,
}

#[cfg(test)]
mod tests {
    use super::Creds;

    #[test]
    fn test_usage() {
        let creds = Creds {
            token: "token".to_string(),
            chat_id: "chat".to_string(),
        };
        assert_eq!(creds.token, "token");
        assert_eq!(creds.chat_id, "chat");
    }
}
