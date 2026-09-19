use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;

use super::constants::MAX_FRAME;

/// # Errors
/// Returns an error if the name is too long or the frame cannot be written.
pub async fn write_hello<W>(stream: &mut W, name: &str) -> std::io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let bytes = name.as_bytes();
    if bytes.len() > MAX_FRAME {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "hello too large",
        ));
    }
    let len = u32::try_from(bytes.len())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    stream.write_u32(len).await?;
    stream.write_all(bytes).await
}

// no test_usage necessary
