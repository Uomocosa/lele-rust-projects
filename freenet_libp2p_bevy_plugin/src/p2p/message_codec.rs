use std::io;

use async_trait::async_trait;
use derive_more::Deref;
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::StreamProtocol;
use libp2p::request_response;

use crate::p2p;

#[derive(Deref)]
pub struct MessageCodec<T: p2p::Message>(std::marker::PhantomData<T>);

impl<T: p2p::Message> Default for MessageCodec<T> {
    fn default() -> Self {
        MessageCodec(std::marker::PhantomData)
    }
}

#[rustfmt::skip]
impl<T: p2p::Message> Clone for MessageCodec<T> {
    fn clone(&self) -> Self {
        MessageCodec(std::marker::PhantomData)
    }
}

#[async_trait]
#[rustfmt::skip]
impl<T: p2p::Message> request_response::Codec for MessageCodec<T> {
    type Protocol = StreamProtocol;
    type Request = p2p::Snapshot<T>;
    type Response = p2p::Snapshot<T>;

    async fn read_request<C>(
        &mut self,
        _protocol: &StreamProtocol,
        io: &mut C,
    ) -> io::Result<p2p::Snapshot<T>>
    where
        C: AsyncRead + Unpin + Send,
    {
        let bytes = read_length_prefixed(io).await?;
        decode_snapshot(&bytes)
    }

    async fn read_response<C>(
        &mut self,
        _protocol: &StreamProtocol,
        io: &mut C,
    ) -> io::Result<p2p::Snapshot<T>>
    where
        C: AsyncRead + Unpin + Send,
    {
        let bytes = read_length_prefixed(io).await?;
        decode_snapshot(&bytes)
    }

    async fn write_request<C>(
        &mut self,
        _protocol: &StreamProtocol,
        io: &mut C,
        request: p2p::Snapshot<T>,
    ) -> io::Result<()>
    where
        C: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, &encode_snapshot(&request)?).await
    }

    async fn write_response<C>(
        &mut self,
        _protocol: &StreamProtocol,
        io: &mut C,
        response: p2p::Snapshot<T>,
    ) -> io::Result<()>
    where
        C: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, &encode_snapshot(&response)?).await
    }
}

// needed helper:
async fn read_length_prefixed<C>(io: &mut C) -> io::Result<Vec<u8>>
where
    C: AsyncRead + Unpin + Send,
{
    let mut len_buf = [0u8; 4];
    io.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    io.read_exact(&mut buf).await?;
    Ok(buf)
}

// needed helper:
async fn write_length_prefixed<C>(io: &mut C, bytes: &[u8]) -> io::Result<()>
where
    C: AsyncWrite + Unpin + Send,
{
    let len = (bytes.len() as u32).to_be_bytes();
    io.write_all(&len).await?;
    io.write_all(bytes).await?;
    Ok(())
}

// needed helper:
fn encode_snapshot<T: p2p::Message>(snapshot: &p2p::Snapshot<T>) -> io::Result<Vec<u8>> {
    bincode::serialize(snapshot)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

// needed helper:
fn decode_snapshot<T: p2p::Message>(bytes: &[u8]) -> io::Result<p2p::Snapshot<T>> {
    bincode::deserialize(bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use futures::io::Cursor;
    use libp2p::request_response::Codec;
    use serde::{Deserialize, Serialize};

    use super::MessageCodec;
    use crate::net_id;
    use crate::p2p;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    #[allow(dead_code)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let snapshot = p2p::Snapshot {
            from_id: net_id::NetworkId(1),
            tick: 5,
            sent_at_ms: 100,
            payload: Dummy(3),
        };
        let protocol = libp2p::StreamProtocol::new(p2p::constants::PROTOCOL_NAME);

        let mut codec = MessageCodec::<Dummy>::default();
        let mut buf = Cursor::new(Vec::new());
        let write =
            futures::executor::block_on(codec.write_request(&protocol, &mut buf, snapshot.clone()));
        assert!(write.is_ok());

        buf.set_position(0);
        let read = futures::executor::block_on(codec.read_request(&protocol, &mut buf));
        assert_eq!(read.ok(), Some(snapshot));
    }
}
