use crate::BinaryStream;

use super::MAGIC;

#[derive(Debug)]
pub struct UnconnectedPong {
    pub timestamp: u64,
    pub guid: u64,
    pub magic: [u8; 16],
    pub message: String
}

impl UnconnectedPong {
    pub const ID: u8 = 0x1C;

    pub fn new(timestamp: u64, guid: u64, message: String) -> Self {
        Self { timestamp, guid, magic: MAGIC.to_vec().try_into().unwrap(), message }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        stream.write_long(self.timestamp.try_into().unwrap(), None);
        stream.write((&self.magic).to_vec());
        stream.write_long(self.guid.try_into().unwrap(), None);
        stream.write_string16(&self.message, None);
        stream.binary
    }


    pub fn deserialize(data: Vec<u8>) -> Result<Self, String> {
        let mut stream = BinaryStream::new(Some(data), None);
        stream.skip(1);
        Ok(Self { 
            timestamp: stream.read_long(None).try_into().unwrap(), 
            guid: stream.read_long(None).try_into().unwrap(), 
            magic: stream.read(16).try_into().unwrap(), 
            message: stream.read_string16(None)? 
        })
    }
}

