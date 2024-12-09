use crate::{BinaryStream, Endianness};

use super::Frame;

#[derive(Debug)]
pub struct FrameSet {
    pub sequence: u32,
    pub frames: Vec<Frame>
}

impl FrameSet {
    pub const ID: u8 = 0x80;
    pub fn new(sequence: u32, frames: Vec<Frame>) -> Self {
        Self { sequence, frames }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        stream.write_u24(self.sequence, Some(Endianness::Little));
        for frame in &self.frames {
            frame.write(&mut stream);
        }
        stream.binary
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self, String> {
        let mut stream: BinaryStream = BinaryStream::new(Some(buffer.to_vec()), Some(0));
        stream.skip(1);
        let sequence = stream.read_u24(Some(Endianness::Little));
        let mut frames = Vec::new();
        while !stream.cursor_at_end() {
            let frame = Frame::read(&mut stream);
            if frame.payload.len() == 0 {
                break;
            }
            frames.push(frame);
        }
        Ok(Self { sequence, frames })
    }
}