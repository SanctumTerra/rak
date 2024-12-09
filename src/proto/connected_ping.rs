use crate::BinaryStream;

#[derive(Debug, Clone)]
pub struct ConnectedPing {
    pub timestamp: i64,
}

impl ConnectedPing {
    pub const ID: u8 = 0x00;

    pub fn serialize(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        stream.write_long(self.timestamp, None);
        stream.binary
    }

    pub fn deserialize(binary: &[u8]) -> Self {
        let mut stream = BinaryStream::new(Some(binary.to_vec()), None);
        stream.skip(1);
        let timestamp = stream.read_long(None);
        Self { timestamp }
    }
}


