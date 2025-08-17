use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use serde_json::Value;

pub async fn send_msg(stream: &mut TcpStream, obj: &Value) -> tokio::io::Result<()> {
    let data = serde_json::to_vec(obj).unwrap();
    let len = (data.len() as u32).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(&data).await?;
    Ok(())
}

pub async fn recv_exact(stream: &mut TcpStream, n: usize) -> tokio::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; n];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}

pub async fn recv_msg(stream: &mut TcpStream) -> tokio::io::Result<Value> {
    let len_bytes = recv_exact(stream, 4).await?;
    let len = u32::from_be_bytes(len_bytes.try_into().unwrap()) as usize;
    let payload = recv_exact(stream, len).await?;
    let v: Value = serde_json::from_slice(&payload).map_err(|_| tokio::io::Error::new(tokio::io::ErrorKind::InvalidData, "invalid JSON"))?;
    Ok(v)
}
