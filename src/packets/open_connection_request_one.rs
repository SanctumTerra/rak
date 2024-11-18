use std::io::Write;

use crate::packets::packet::{Packet, PacketError};
use crate::binarystream::BinaryStream;
use super::MAGIC;

#[derive(Debug)]
pub struct OpenConnectionRequestOne {
    pub protocol_version: u8,
    pub mtu_size: usize,
}

impl Packet for OpenConnectionRequestOne {
    const ID: u16 = 0x05;
    const MAGIC: [u8; 16] = MAGIC;

    fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID as u8);    
        stream.write_magic(Self::MAGIC.to_vec());
        stream.write_u8(self.protocol_version);
        let udp_overhead = 28;
        let current_size = stream.binary.len();
        let padding_size = self.mtu_size - udp_overhead - current_size;
        stream.binary.write(&vec![0; padding_size]).map_err(|_| PacketError::SerializationError)?;
        Ok(stream.binary)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self, PacketError> {
        let mut stream = BinaryStream::from(buffer.to_vec(), Some(0));
        if buffer.len() < 19 {
            return Err(PacketError::InvalidPacket);
        }
        let id = stream.read_u8()?;
        if id != Self::ID as u8 {
            return Err(PacketError::InvalidPacket);
        }
        let magic = stream.read_magic()?;
        if magic != Self::MAGIC {
            return Err(PacketError::InvalidPacket);
        }
        let protocol_version = stream.read_u8()?;
        let udp_overhead = 28;
        let mtu_size = buffer.len() + udp_overhead;
        
        // println!("ID: {}", id);
        // println!("Magic: {:?}", magic);
        // println!("Protocol Version: {}", protocol_version);
        // println!("MTU Size: {}", mtu_size);
        
        Ok(Self {
            protocol_version,
            mtu_size
        })
    }
}