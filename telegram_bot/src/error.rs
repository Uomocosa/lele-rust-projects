use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("telegram {op}: empty payload")]
    EmptyPayload { op: &'static str },
    #[error("telegram read {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("telegram {kind} too small ({bytes} bytes): {path}")]
    TooSmall {
        kind: &'static str,
        path: String,
        bytes: usize,
    },
    #[error("telegram {path} is not png")]
    NotPng { path: String },
    #[error("telegram {path} is not mp4 (no ftyp box)")]
    NotMp4 { path: String },
    #[error("telegram {op} request failed: {message}")]
    Request { op: &'static str, message: String },
    #[error("telegram {op} failed: status={status} body={snippet}")]
    Rejected {
        op: &'static str,
        status: String,
        snippet: String,
    },
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn test_usage() {
        let err = Error::EmptyPayload { op: "send_text" };
        assert_eq!(format!("{err}"), "telegram send_text: empty payload");
        let err = Error::NotPng {
            path: "preview.png".to_string(),
        };
        assert_eq!(format!("{err}"), "telegram preview.png is not png");
        let err = Error::Rejected {
            op: "sendMessage",
            status: "400".to_string(),
            snippet: "bad".to_string(),
        };
        assert!(format!("{err}").contains("status=400"));
    }
}
