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
        let data = from(Error::StdinClosed);
        assert_eq!(data.message, "stdin channel closed");
    }
}
