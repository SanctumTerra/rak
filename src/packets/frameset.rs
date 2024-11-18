use crate::binarystream::{BinaryStream, Endianness};

use super::{ 
    DataType, 
    Frame, 
    Packet, 
    PacketError, 
    MAGIC 
};

pub struct FrameSet {
    pub sequence: u32,
    pub frames: Vec<Frame>
}

impl Packet for FrameSet {
    const ID: u16 = 0x80;
    const MAGIC: [u8; 16] = MAGIC;

    fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID as u8);
        stream.write_u24(self.sequence, Some(Endianness::Little));
        for frame in &self.frames {
            let _ = frame.write(&mut stream);
        }
        Ok(stream.binary)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self, PacketError> {
        let mut stream = BinaryStream::from(buffer.to_vec(), Some(0));
        let _ = stream.read_u8().unwrap();
        let sequence = stream.read_u24(Some(Endianness::Little)).unwrap();
        let mut frames = Vec::<Frame>::new();
        while !stream.cursor_at_end() {
            frames.push(Frame::read(&mut stream));
        }
        Ok(Self { sequence, frames })
    }
}
