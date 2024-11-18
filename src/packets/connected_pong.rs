use crate::binarystream::BinaryStream;
use super::{Packet, PacketError, MAGIC};

#[derive(Debug)]
pub struct ConnectedPong {
    pub ping_time: i64,
    pub pong_time: i64,
}

impl Packet for ConnectedPong {
    const ID: u16 = 0x03;
    const MAGIC: [u8; 16] = MAGIC;

    fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_byte(Self::ID as u8);
        stream.write_long(self.ping_time, None);
        stream.write_long(self.pong_time, None);
        Ok(stream.binary)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self, PacketError> {
        let mut stream = BinaryStream::new(Some(buffer.to_vec()), None);
        stream.read_byte().unwrap();
        let ping_time = stream.read_long(None).unwrap();
        let pong_time = stream.read_long(None).unwrap();
        Ok(ConnectedPong { ping_time, pong_time })
    }
}
