use crate::{BinaryStream, Endianness};

use super::{Flags, Reliability};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Frame {
    pub reliable_frame_index: Option<u32>,
    pub sequence_frame_index: Option<u32>,
    pub ordered_frame_index: Option<u32>,
    pub order_channel: Option<u8>,
    pub reliability: Reliability,
    pub payload: Vec<u8>,
    pub split_frame_index: Option<u32>,
    pub split_id: Option<u16>,
    pub split_size: Option<u32>,
}

impl Frame {
    pub fn is_split(&self) -> bool {
        self.split_size.is_some() && self.split_size.unwrap() > 0
    }

    pub fn get_size(&self) -> usize {
        let mut size = self.payload.len();
        size += 3; // Flags and reliability
        if self.reliability.is_reliable() {
            size += 3; // reliable_frame_index
        }
        if self.reliability.is_sequenced() {
            size += 3; // sequence_frame_index
        }
        if self.reliability.is_ordered() {
            size += 4; // ordered_frame_index + order_channel
        }
        if self.is_split() {
            size += 10; 
        }
        size
    }

    pub fn read(stream: &mut BinaryStream) -> Self {
        let flags = stream.read_u8();
        let reliability = Reliability::from_u8((flags & 0xe0) >> 5);
        let split = (flags & Flags::Split as u8) != 0;
        let bits = stream.read_u16(None);
        let length = ((bits as f32) / 8.0).ceil() as usize;
        let reliable_frame_index = if reliability.is_reliable() { 
            Some(stream.read_u24(Some(Endianness::Little))) 
        } else { 
            None 
        };
        let sequence_frame_index = if reliability.is_sequenced() { Some(stream.read_u24(Some(Endianness::Little))) } else { None };
        let (ordered_frame_index, order_channel) = if reliability.is_ordered() {
            (Some(stream.read_u24(Some(Endianness::Little))), Some(stream.read_u8()))
        } else { (None, None) };
        let (split_size, split_id, split_frame_index) = if split {
            (Some(stream.read_u32(None)), Some(stream.read_u16(None)), Some(stream.read_u32(None)))
        } else { (None, None, None) };
        let payload = stream.read(length as usize);
        Self {
            reliable_frame_index, sequence_frame_index, ordered_frame_index,
            order_channel, reliability, payload,
            split_frame_index, split_id, split_size,
        }
    }


    pub fn write(&self, stream: &mut BinaryStream) {
        let flags = (self.reliability.clone() as u8) << 5 |
            if self.is_split() { Flags::Split as u8 } else { 0 };
        stream.write_u8(flags);
        stream.write_u16((self.payload.len() as u16) << 3, None);
        if self.reliability.is_reliable() {
            stream.write_u24(self.reliable_frame_index.unwrap(), Some(Endianness::Little));
        }
        if self.reliability.is_sequenced() {
            stream.write_u24(self.sequence_frame_index.unwrap(), Some(Endianness::Little));
        }
        if self.reliability.is_ordered() {
            stream.write_u24(self.ordered_frame_index.unwrap(), Some(Endianness::Little));
            stream.write_u8(self.order_channel.unwrap());
        }
        if self.is_split() {
            stream.write_u32(self.split_size.unwrap(), None);
            stream.write_u16(self.split_id.unwrap(), None);
            stream.write_u32(self.split_frame_index.unwrap(), None);
        }

        stream.write(self.payload.clone());
    }

    pub fn new() -> Self {
        Self {
            reliable_frame_index: None, 
            sequence_frame_index: None, 
            ordered_frame_index: None, 
            order_channel: Some(0), 
            reliability: Reliability::ReliableOrdered, 
            payload: Vec::new(), 
            split_frame_index: None, 
            split_id: None, 
            split_size: None,
        }
    }
}
