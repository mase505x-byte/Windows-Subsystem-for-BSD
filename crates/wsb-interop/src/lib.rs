//! Length-prefixed client for the local WSB host bridge.
//! Each request uses a fresh connection, bounded frames, and an optional socket-path override.
use std::io::{Read, Write};
use std::time::Duration;
use wsb_core::{ServerRequest, ServerResponse, DEFAULT_SOCKET_PATH, MAX_FRAME_SIZE};

#[cfg(unix)]
pub fn send_command(request: &ServerRequest) -> Result<ServerResponse, String> {
    use std::os::unix::net::UnixStream;
    let path = std::env::var_os("WSB_SOCKET").unwrap_or_else(|| DEFAULT_SOCKET_PATH.into());
    let mut stream = UnixStream::connect(path).map_err(|e| format!("could not connect to WSB bridge: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(5))).map_err(|e| e.to_string())?;
    let payload = bincode::serialize(request).map_err(|e| format!("encode request: {e}"))?;
    if payload.is_empty() || payload.len() > MAX_FRAME_SIZE {
        return Err("request frame exceeds the 1 MiB protocol limit".into());
    }
    stream.write_all(&(payload.len() as u32).to_le_bytes()).map_err(|e| format!("write frame header: {e}"))?;
    stream.write_all(&payload).map_err(|e| format!("write request: {e}"))?;
    let mut header = [0u8; 4];
    stream.read_exact(&mut header).map_err(|e| format!("read frame header: {e}"))?;
    let size = u32::from_le_bytes(header) as usize;
    if size == 0 || size > MAX_FRAME_SIZE {
        return Err("server response frame exceeds the 1 MiB protocol limit".into());
    }
    let mut response = vec![0; size];
    stream.read_exact(&mut response).map_err(|e| format!("read response: {e}"))?;
    bincode::deserialize(&response).map_err(|e| format!("decode response: {e}"))
}

#[cfg(not(unix))]
pub fn send_command(_request: &ServerRequest) -> Result<ServerResponse, String> {
    Err("the Unix-domain-socket bridge client is only available on Unix targets".into())
}

#[no_mangle]
pub extern "C" fn wsb_ping_server() -> i32 {
    match send_command(&ServerRequest::Ping) {
        Ok(ServerResponse::Pong) => 0,
        _ => -1,
    }
}

#[no_mangle]
pub extern "C" fn wsb_protocol_version() -> u16 {
    wsb_core::PROTOCOL_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ffi_version_is_stable() { assert_eq!(wsb_protocol_version(), 1); }
}
