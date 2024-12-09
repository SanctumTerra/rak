use crate::BinaryStream;

use super::Address;

#[derive(Debug, Clone)]
pub struct ConnectionRequestAccepted {
    pub client_address: Address,
    pub client_id: u16, 
    pub server_addresses: Vec<Address>,
    pub client_send_time: i64,
    pub server_send_time: i64
}

impl ConnectionRequestAccepted {
    pub const ID: u8 = 0x10;

    pub fn new(client_address: Address, client_id: u16, server_addresses: Vec<Address>, client_send_time: i64, server_send_time: i64) -> Self {
        Self { client_address, client_id, server_addresses, client_send_time, server_send_time }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        // TODO: Implement
        stream.binary
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self, String> {
        let mut stream = BinaryStream::new(Some(buffer.to_vec()), Some(0));
        stream.skip(1);
        let client_address = Address::read(&mut stream);
        let client_id = stream.read_u16(None);
        let mut server_addresses = Vec::new();
        for _ in 0..20 {
            server_addresses.push(Address::read(&mut stream));
        }        
        let client_send_time = stream.read_long(None);
        let server_send_time = stream.read_long(None);
        Ok(Self::new(client_address, client_id, server_addresses, client_send_time, server_send_time))
    }
}