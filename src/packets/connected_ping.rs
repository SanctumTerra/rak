use crate::binarystream::BinaryStream;

use super::{Packet, PacketError, MAGIC};

#[derive(Debug)]
pub struct ConnectedPing {
    pub timestamp: i64,
}

impl Packet for ConnectedPing {
    const ID: u16 = 0x00;
    const MAGIC: [u8; 16] = MAGIC;

    fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_byte(Self::ID as u8);
        stream.write_long(self.timestamp, None);
        Ok(stream.binary)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self, PacketError> {
        let mut stream = BinaryStream::new(Some(buffer.to_vec()), None);
        stream.read_byte().unwrap();
        let timestamp = stream.read_long(None).unwrap();
        Ok(ConnectedPing { timestamp })
    }
}
