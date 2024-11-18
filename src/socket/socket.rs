use std::net::{SocketAddr, UdpSocket};

pub struct Socket {
    pub bind_address: SocketAddr,
    pub socket: UdpSocket,
}

impl Clone for Socket {
    fn clone(&self) -> Self {
        Self {
            bind_address: self.bind_address,
            socket: self.socket.try_clone().expect("Failed to clone socket"),
        }
    }
}

#[derive(Debug)]
pub enum SocketError {
    BindError(#[allow(dead_code)] std::io::Error),
    SendError(#[allow(dead_code)] std::io::Error),
    ReceiveError(#[allow(dead_code)] std::io::Error),
    AddressParseError(#[allow(dead_code)] std::io::Error),
}

impl Socket {
    pub fn new(bind_address: Option<String>, port: Option<u16>) -> Result<Self, SocketError> {
        let bind_addr = match (bind_address, port) {
            (Some(addr), Some(p)) => format!("{}:{}", addr, p),
            (Some(addr), None) => addr,
            (None, Some(p)) => format!("0.0.0.0:{}", p),
            (None, None) => "0.0.0.0:0".to_string(),
        };

        let socket = UdpSocket::bind(bind_addr).map_err(SocketError::BindError)?;
        let bind_address = socket.local_addr().map_err(SocketError::BindError)?;
        
        Ok(Self { 
            bind_address,
            socket 
        })
    }

    pub fn send(&self, buffer: &[u8], send_address: &str) -> Result<usize, SocketError> {
        let addr: SocketAddr = send_address
            .parse()
            .map_err(|e| SocketError::AddressParseError(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;
        
        self.socket.send_to(buffer, addr).map_err(SocketError::SendError)
    }

    pub fn receive(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr), SocketError> {
        self.socket
            .recv_from(buffer)
            .map_err(SocketError::ReceiveError)
    }

    pub fn get_local_port(&self) -> u16 {
        self.bind_address.port()
    }
}
