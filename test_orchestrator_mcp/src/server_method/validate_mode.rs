use crate::Error;

pub fn validate_mode(mode: &str) -> Result<(), Error> {
    match mode {
        "test" | "build" | "release" | "release-notests" => Ok(()),
        other => Err(Error::InvalidMode(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::validate_mode;

    #[test]
    fn test_usage() {
        assert!(validate_mode("release").is_ok());
        assert!(validate_mode("nope").is_err());
    }
}
