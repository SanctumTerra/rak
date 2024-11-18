use crate::binarystream::BinaryStream;

use super::{Packet, PacketError, MAGIC};

pub struct UnconnectedPing {
    timestamp: i64,
    guid: i64,
}

impl UnconnectedPing {
    pub fn new(timestamp: i64, guid: i64) -> Self {
        Self {
            timestamp,
            guid,
        }
    }
}

impl Packet for UnconnectedPing {
    const ID: u16 = 0x01;
    const MAGIC: [u8; 16] = MAGIC;

    fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID as u8);
        stream.write_long(self.timestamp, None);
        stream.write_magic(Self::MAGIC.to_vec());
        stream.write_long(self.guid, None);
        Ok(stream.binary)
    }

    fn deserialize(data: &[u8]) -> Result<Self, PacketError> {
        let mut stream = BinaryStream::new(Some(data.to_vec()), None);
        stream.read_u8()?;
        let timestamp = stream.read_long(None)?;
        stream.read_magic()?;
        let guid = stream.read_long(None)?;
        Ok(Self::new(timestamp, guid))
    }
}

