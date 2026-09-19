use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;

use super::envelope::Envelope;

/// # Errors
/// Returns an error if the envelope cannot be serialized or the frame cannot
/// be written to the stream.
pub async fn write_event<W>(stream: &mut W, envelope: &Envelope) -> std::io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let bytes = bincode::serialize(envelope)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let len = u32::try_from(bytes.len())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    stream.write_u32(len).await?;
    stream.write_all(&bytes).await
}

// no test_usage necessary
