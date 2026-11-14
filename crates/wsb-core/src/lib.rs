//! Shared protocol and configuration primitives for WSB.
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub const PROTOCOL_VERSION: u16 = 1;
pub const DEFAULT_SOCKET_PATH: &str = "/var/run/wsb_server.sock";
pub const MAX_FRAME_SIZE: usize = 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerRequest {
    Ping,
    ExecuteWindowsBinary { path: String, args: Vec<String> },
    MountDrive { drive_letter: char, mount_point: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerResponse {
    Pong,
    Success(i32),
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerConfig {
    pub bind: SocketAddr,
}

impl ServerConfig {
    /// Validate user-supplied bind settings without opening a socket or changing the host.
    pub fn parse(bind: &str, port: u16) -> Result<Self, String> {
        if port == 0 {
            return Err("port must be between 1 and 65535".into());
        }
        let address: std::net::IpAddr = bind.parse().map_err(|_| "bind must be an IPv4 or IPv6 address")?;
        Ok(Self { bind: SocketAddr::new(address, port) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_bind_address_and_port() {
        assert_eq!(ServerConfig::parse("127.0.0.1", 9000).unwrap().bind.port(), 9000);
        assert!(ServerConfig::parse("not-an-ip", 9000).is_err());
        assert!(ServerConfig::parse("127.0.0.1", 0).is_err());
    }
}
