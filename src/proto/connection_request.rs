use crate::BinaryStream;

pub struct ConnectionRequest { 
    pub guid: i64,
    pub timestamp: i64,
    pub security: bool,
}

impl ConnectionRequest {
    pub const ID: u8 = 0x09;

    pub fn new(guid: i64, timestamp: i64, security: bool) -> Self {
        Self { guid, timestamp, security }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        stream.write_long(self.guid, None);
        stream.write_long(self.timestamp, None);
        stream.write_bool(self.security);
        stream.binary
    }

    pub fn deserialize(binary: &[u8]) -> Self {
        let mut stream = BinaryStream::new(Some(binary.to_vec()), None);
        stream.skip(1);
        let guid = stream.read_long(None);
        let timestamp = stream.read_long(None);
        let security = stream.read_bool();
        Self::new(guid, timestamp, security)
    }
}
