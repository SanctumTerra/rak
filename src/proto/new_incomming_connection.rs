use crate::BinaryStream;

use super::Address;

#[derive(Debug, Clone)]
pub struct NewIncommingConnection {
    pub server_address: Address,
    pub internal_addresses: [Address; 20],
    pub incoming_timestamp: i64,
    pub server_timestamp: i64
}

impl NewIncommingConnection {
    pub const ID: u8 = 0x13;

    pub fn new(server_address: Address, internal_addresses: [Address; 20], incoming_timestamp: i64, server_timestamp: i64) -> Self {
        Self { server_address, internal_addresses, incoming_timestamp, server_timestamp }
    }   

    pub fn serialize(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        self.server_address.write(&mut stream);
        for i in 0..20 {
            self.internal_addresses[i].write(&mut stream);
        }
        stream.write_long(self.incoming_timestamp, None);
        stream.write_long(self.server_timestamp, None);
        stream.binary
    }

    pub fn deserialize(stream: &mut BinaryStream) -> Self {
        let server_address = Address::read(stream);
        let internal_addresses = core::array::from_fn(|_| Address::read(stream));
        let incoming_timestamp = stream.read_long(None);
        let server_timestamp = stream.read_long(None);
        Self::new(server_address, internal_addresses, incoming_timestamp, server_timestamp)
    }
}
