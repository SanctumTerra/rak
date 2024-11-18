use crate::binarystream::{BinaryStream, Endianness};

use super::{Address, DataType, Packet, PacketError, MAGIC};

#[derive(Debug)]
pub struct ConnectionRequestAccepted {
    pub client_address: Address,
    pub client_id: u16, 
    pub server_addresses: Vec<Address>,
    pub client_send_time: i64,
    pub server_send_time: i64,
}

impl Packet for ConnectionRequestAccepted {
    const ID: u16 = 0x10;
    const MAGIC: [u8; 16] = MAGIC;

    fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID as u8);
        self.client_address.write(&mut stream)?;
        stream.write_short(self.client_id as u16, None);
        for address in &self.server_addresses {
            address.write(&mut stream)?;
        }
        stream.write_long(self.client_send_time, None);
        stream.write_long(self.server_send_time, None);
        Ok(stream.binary)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self, PacketError> {
        let mut stream = BinaryStream::from(buffer.to_vec(), None);
        let _ = stream.read_byte().unwrap();

        let client_address = Address::read(&mut stream);
        let client_id = stream.read_short(None).unwrap();

        let mut server_addresses = Vec::new();

        for _ in 0..20 {
            server_addresses.push(Address::read(&mut stream));
        }

        let client_send_time = stream.read_long(None).unwrap();
        let server_send_time = stream.read_long(None).unwrap();
        
        Ok(Self {
            client_address,
            client_id,
            server_addresses,
            client_send_time,
            server_send_time,
        })
    }
}
