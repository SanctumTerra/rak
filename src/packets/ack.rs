use crate::binarystream::{BinaryStream, Endianness};

use super::{Packet, PacketError, MAGIC};

#[derive(Debug)]
pub struct Ack {
    pub sequences: Vec<u32>,
}

impl Packet for Ack {
    const ID: u16 = 0xc0;
    const MAGIC: [u8; 16] = MAGIC;

    fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID as u8);
        let mut _stream = BinaryStream::new(None, None);
        let count = self.sequences.len() as u16;
        let mut records = 0;

        if count > 0 {
            let mut cursor = 0;
            let mut start = self.sequences[cursor];
            let mut last = self.sequences[cursor];
            while cursor < (count as usize - 1) {
                let current = self.sequences[{cursor += 1; cursor}];
                let diff = current - last;

                if diff == 1 {
                    last = current;
                } else if diff > 1 {
                    if start == last {
                        _stream.write_bool(true);
                        _stream.write_u24(start, Some(Endianness::Little));
                    } else {
                        _stream.write_bool(false);
                        _stream.write_u24(start, Some(Endianness::Little));
                        _stream.write_u24(last, Some(Endianness::Little));
                    }
                    start = current;
                    last = current;
                    records += 1;
                }
            }

            if start == last { 
                _stream.write_bool(true);
                _stream.write_u24(start, Some(Endianness::Little));
            } else {
                _stream.write_bool(false);
                _stream.write_u24(start, Some(Endianness::Little));
                _stream.write_u24(last, Some(Endianness::Little));
            }

            records += 1;
            stream.write_u16(records, None);
            stream.write(_stream.binary);
        }
        Ok(stream.binary)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self, PacketError> {
        let mut stream = BinaryStream::from(buffer.to_vec(), None);
        let _id = stream.read_u8().unwrap();
        let mut sequences = Vec::new();
        let records = stream.read_u16(None).unwrap();
        for _ in 0..records {
            let range = stream.read_bool().unwrap();
            if range {
                let r: u32 = stream.read_u24(Some(Endianness::Little)).unwrap();
                sequences.push(r);
            } else {
                let r = stream.read_u24(Some(Endianness::Little)).unwrap();
                let l = stream.read_u24(Some(Endianness::Little)).unwrap();
                for i in r..=l {
                    sequences.push(i);
                }
            }
        }
        Ok(Self { sequences })
    }
}
