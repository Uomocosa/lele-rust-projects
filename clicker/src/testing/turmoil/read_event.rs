use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;

use super::constants::MAX_FRAME;
use super::envelope::Envelope;

/// # Errors
/// Returns an error if the frame length is invalid, exceeds `MAX_FRAME`, the
/// stream read fails, or the payload cannot be deserialized.
pub async fn read_event<R>(stream: &mut R) -> std::io::Result<Envelope>
where
    R: AsyncRead + Unpin,
{
    let len = stream.read_u32().await?;
    let len = usize::try_from(len)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if len > MAX_FRAME {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame too large",
        ));
    }
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    bincode::deserialize(&buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

// no test_usage necessary
