use crate::binarystream::{BinaryStream, Endianness};

use super::{Address, DataType, Packet, PacketError, MAGIC};

#[derive(Debug)]
pub struct NewIncomingConnection {
    pub server_address: Address,
    pub client_addresses: Address,
    pub client_send_timestamp: i64,
    pub server_send_timestamp: i64,
}

impl Packet for NewIncomingConnection {
    const ID: u16 = 0x13;
    const MAGIC: [u8; 16] = MAGIC;

    fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID as u8);

        self.server_address.write(&mut stream)?;

        for _ in 0..20 {
            self.client_addresses.clone().write(&mut stream)?;
        }

        stream.write_long(self.client_send_timestamp, None);
        stream.write_long(self.server_send_timestamp, None);

        Ok(stream.binary)
    }

    fn deserialize(data: &[u8]) -> Result<Self, PacketError> {
        let mut stream = BinaryStream::new(Some(data.to_vec()), None);
        let _id = stream.read_u8()?;

        let server_address = Address::read(&mut stream);
        let mut client_addresses = Address::new("0.0.0.0".to_string(), 0, 4);

        for _ in 0..20 {
            client_addresses = Address::read(&mut stream);
        }

        let client_send_timestamp = stream.read_long(None)?;
        let server_send_timestamp = stream.read_long(None)?;

        Ok(Self { server_address, client_addresses, client_send_timestamp, server_send_timestamp })
    }
}
