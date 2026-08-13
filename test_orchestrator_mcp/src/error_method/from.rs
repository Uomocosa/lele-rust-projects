use rmcp::ErrorData;

use crate::Error;

pub fn from(value: Error) -> ErrorData {
    ErrorData::internal_error(value.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::from;
    use crate::Error;

    #[test]
    fn test_usage() {
        let data = from(Error::InvalidMode("x".into()));
        assert_eq!(
            data.message,
            "invalid mode 'x', expected one of test, build, release, release-notests"
        );
    }
}
