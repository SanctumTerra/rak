use crate::BinaryStream;

#[derive(Debug, Clone)]
pub struct ConnectedPong {
    pub ping_timestamp: i64,
    pub pong_timestamp: i64,
}

impl ConnectedPong {
    pub const ID: u8 = 0x03;

    pub fn new(ping_timestamp: i64, pong_timestamp: i64) -> Self {
        Self { ping_timestamp, pong_timestamp }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        stream.write_long(self.ping_timestamp, None);
        stream.write_long(self.pong_timestamp, None);
        stream.binary
    }

    pub fn deserialize(binary: &[u8]) -> Self {
        let mut stream = BinaryStream::new(Some(binary.to_vec()), None);
        stream.skip(1);
        let ping_timestamp = stream.read_long(None);
        let pong_timestamp = stream.read_long(None);
        Self { ping_timestamp, pong_timestamp }
    }
}

