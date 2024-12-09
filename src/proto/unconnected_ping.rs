use crate::BinaryStream;

use super::MAGIC;

pub struct UnconnectedPing {
    pub timestamp: i64,
    pub magic: [u8; 16],
    pub guid: i64
}

impl UnconnectedPing {
    pub const ID: u8 = 0x01;

    pub fn new(timestamp: i64, guid: i64) -> Self {
        Self { timestamp, magic: MAGIC.to_vec().try_into().unwrap(), guid }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        stream.write_long(self.timestamp.try_into().unwrap(), None);
        stream.write((&self.magic).to_vec());
        stream.write_long(self.guid, None);
        stream.binary
    }

    pub fn deserialize(data: &[u8]) -> Self {
        let mut stream = BinaryStream::new(Some(data.to_vec()), None);
        let _id = stream.read_u8();
        let timestamp = stream.read_long(None).try_into().unwrap();
        let magic = stream.read(16);
        let guid = stream.read_long(None).try_into().unwrap();

        Self { timestamp, magic: magic.try_into().unwrap(), guid }
    }
}
