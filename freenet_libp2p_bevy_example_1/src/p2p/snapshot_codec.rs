use std::io;

use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::StreamProtocol;
use libp2p::request_response;

use crate::p2p;

#[derive(Clone)]
pub struct SnapshotCodec;

#[async_trait]
#[rustfmt::skip]
impl request_response::Codec for SnapshotCodec {
    type Protocol = StreamProtocol;
    type Request = p2p::Snapshot;
    type Response = p2p::Snapshot;

    async fn read_request<T>(
        &mut self,
        _protocol: &StreamProtocol,
        io: &mut T,
    ) -> io::Result<p2p::Snapshot>
    where
        T: AsyncRead + Unpin + Send,
    {
        let bytes = read_length_prefixed(io).await?;
        decode_snapshot(&bytes)
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &StreamProtocol,
        io: &mut T,
    ) -> io::Result<p2p::Snapshot>
    where
        T: AsyncRead + Unpin + Send,
    {
        let bytes = read_length_prefixed(io).await?;
        decode_snapshot(&bytes)
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &StreamProtocol,
        io: &mut T,
        request: p2p::Snapshot,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, &encode_snapshot(&request)?).await
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &StreamProtocol,
        io: &mut T,
        response: p2p::Snapshot,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, &encode_snapshot(&response)?).await
    }
}

// needed helper:
async fn read_length_prefixed<T>(io: &mut T) -> io::Result<Vec<u8>>
where
    T: AsyncRead + Unpin + Send,
{
    let mut len_buf = [0u8; 4];
    io.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    io.read_exact(&mut buf).await?;
    Ok(buf)
}

// needed helper:
async fn write_length_prefixed<T>(io: &mut T, bytes: &[u8]) -> io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
{
    let len = (bytes.len() as u32).to_be_bytes();
    io.write_all(&len).await?;
    io.write_all(bytes).await?;
    Ok(())
}

// needed helper:
fn encode_snapshot(snapshot: &p2p::Snapshot) -> io::Result<Vec<u8>> {
    bincode::serialize(snapshot)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

// needed helper:
fn decode_snapshot(bytes: &[u8]) -> io::Result<p2p::Snapshot> {
    bincode::deserialize(bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

#[cfg(test)]
mod tests {
    use futures::io::Cursor;
    use libp2p::request_response::Codec;

    use super::SnapshotCodec;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let snapshot = p2p::Snapshot {
            player_id: 1,
            x: 10.0,
            y: 20.0,
            vx: 1.0,
            vy: 0.0,
            tick: 5,
            sent_at_ms: 100,
        };
        let protocol = libp2p::StreamProtocol::new(p2p::constants::PROTOCOL_NAME);

        let mut codec = SnapshotCodec;
        let mut buf = Cursor::new(Vec::new());
        let write = futures::executor::block_on(codec.write_request(&protocol, &mut buf, snapshot));
        assert!(write.is_ok());

        buf.set_position(0);
        let read = futures::executor::block_on(codec.read_request(&protocol, &mut buf));
        assert_eq!(read.ok(), Some(snapshot));
    }
}
