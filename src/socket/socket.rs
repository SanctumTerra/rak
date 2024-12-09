use std::net::{SocketAddr, UdpSocket};

#[derive(Debug)]
pub struct Socket {
    pub socket: UdpSocket,
    pub server_address: String,
    pub server_port: u16,
}

impl Socket { 
    pub fn new(server_address: String, server_port: u16) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.connect(format!("{}:{}", server_address, server_port)).unwrap();
        
        Self { 
            socket,
            server_address,
            server_port,
        }
    }

    pub fn send(&self, data: Vec<u8>) -> Result<usize, std::io::Error> {
        self.socket.send(&data)
    }

    pub fn receive(&self, buffer: &mut [u8]) -> Result<usize, std::io::Error> {
        self.socket.set_nonblocking(true)?;
        match self.socket.recv(buffer) {
            Ok(size) => Ok(size),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(e)
        }
    }

    pub fn get_address(&self) -> SocketAddr {
        self.socket.local_addr().unwrap()
    }
}

