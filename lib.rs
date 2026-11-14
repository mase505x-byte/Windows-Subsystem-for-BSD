use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Mutex, OnceLock};

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerRequest {
    ExecuteWindowsBinary { path: String, args: Vec<String> },
    MountDrive { drive_letter: char, mount_point: String },
    Ping,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerResponse {
    Success(i32),
    Error(String),
    Pong,
}

static SERVER_CONN: OnceLock<Mutex<UnixStream>> = OnceLock::new();

fn get_connection() -> Result<&'static Mutex<UnixStream>, String> {
    let stream = UnixStream::connect("/var/run/wsb_server.sock")
        .map_err(|e| format!("Failed to reach WSB server binary: {}", e))?;
    Ok(SERVER_CONN.get_or_init(|| Mutex::new(stream)))
}

pub fn send_command(req: ServerRequest) -> Result<ServerResponse, String> {
    let conn_mutex = get_connection()?;
    let mut stream = conn_mutex.lock().map_err(|_| "Lock poisoned")?;

    let payload = bincode::serialize(&req).map_err(|e| e.to_string())?;
    let len = (payload.len() as u32).to_le_bytes();

    stream.write_all(&len).map_err(|e| e.to_string())?;
    stream.write_all(&payload).map_err(|e| e.to_string())?;

    let mut resp_len_buf = [0u8; 4];
    stream.read_exact(&mut resp_len_buf).map_err(|e| e.to_string())?;
    let resp_len = u32::from_le_bytes(resp_len_buf) as usize;

    let mut resp_buf = vec![0u8; resp_len];
    stream.read_exact(&mut resp_buf).map_err(|e| e.to_string())?;

    bincode::deserialize(&resp_buf).map_err(|e| e.to_string())
}

#[no_mangle]
pub extern "C" fn wsb_ping_server() -> i32 {
    match send_command(ServerRequest::Ping) {
        Ok(ServerResponse::Pong) => 0,
        _ => -1,
    }
}
