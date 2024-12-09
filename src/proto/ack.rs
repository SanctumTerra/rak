use crate::{BinaryStream, Endianness};

#[derive(Debug)]
pub struct Ack {
    pub sequences: Vec<u32>
}

impl Ack {
    pub const ID: u8 = 0xc0;

    pub fn new(sequences: Vec<u32>) -> Self {
        Self { sequences }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        let mut _stream = BinaryStream::new(None, None);
        
        let mut sequences = self.sequences.clone();
        sequences.sort();
        
        let count = sequences.len() as u16;
        let mut records = 0;

        if count > 0 {
            let mut pointer = 1;
            let mut start = sequences[0];
            let mut last = sequences[0];
            
            while pointer < count as usize {
                let current = sequences[pointer];
                pointer += 1;
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
        } else {
            stream.write_u16(0, None);
        }
        
        stream.binary
    }

    pub fn deserialize(buffer: &[u8]) -> Self {
        let mut stream = BinaryStream::new(Some(buffer.to_vec()), None);
        let _id = stream.read_u8();
        let mut sequences = Vec::new();
        let records = stream.read_u16(None);
        for _ in 0..records {
            let range = stream.read_bool();
            if range {
                let r = stream.read_u24(Some(Endianness::Little));
                sequences.push(r);
            } else {
                let r = stream.read_u24(Some(Endianness::Little));
                let l = stream.read_u24(Some(Endianness::Little));
                for i in r..=l {
                    sequences.push(i);
                }
            }
        }
        Self { sequences }
    }
}
