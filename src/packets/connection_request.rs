use crate::binarystream::BinaryStream;

use super::{PacketError, MAGIC};

#[derive(Debug)]
pub struct ConnectionRequest {
    pub guid: i64,
    pub timestamp: i64,
    pub security: bool,
}

impl ConnectionRequest {
    const ID: u16 = 0x09;
    const MAGIC: [u8; 16] = MAGIC;  

    pub fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID as u8);
        stream.write_long(self.guid, None);
        stream.write_long(self.timestamp, None);
        stream.write_bool(self.security);
        Ok(stream.binary)
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self, PacketError> {
        let mut stream = BinaryStream::from(buffer.to_vec(), Some(0));
        let _ = stream.read_byte().unwrap();
        Ok(Self {
            guid: stream.read_long(None).unwrap(),
            timestamp: stream.read_long(None).unwrap(),
            security: stream.read_bool().unwrap(),
        })
    }
}
