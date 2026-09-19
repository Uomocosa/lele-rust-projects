use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

use super::constants::MAX_FRAME;

/// # Errors
/// Returns an error if the frame length is invalid, exceeds `MAX_FRAME`, the
/// stream read fails, or the payload is not valid UTF-8.
pub async fn read_hello<R>(stream: &mut R) -> std::io::Result<String>
where
    R: AsyncRead + Unpin,
{
    let len = stream.read_u32().await?;
    let len = usize::try_from(len)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if len > MAX_FRAME {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "hello frame too large",
        ));
    }
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    String::from_utf8(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

// no test_usage necessary
