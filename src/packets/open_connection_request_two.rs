use crate::{binarystream::BinaryStream, packets::types::Address};

use super::{DataType, Packet, PacketError, MAGIC};

pub struct OpenConnectionRequestTwo {
    pub address: Address,
    pub mtu_size: u16,
    pub guid: i64,
}

impl Packet for OpenConnectionRequestTwo {
    const ID: u16 = 0x07;
    const MAGIC: [u8; 16] = MAGIC;
    
    fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID as u8);
        stream.write_magic(Self::MAGIC.to_vec());
        self.address.write(&mut stream)?;
        stream.write_u16(self.mtu_size, None);
        stream.write_long(self.guid, None);
        Ok(stream.binary)
    }

    fn deserialize(data: &[u8]) -> Result<Self, PacketError> {
        let mut stream = BinaryStream::new(Some(data.to_vec()), None);
        stream.read_u8()?;
        stream.read_magic()?;
        let address = Address::read(&mut stream);
        let mtu_size = stream.read_u16(None)?;
        let guid = stream.read_long(None)?;
        Ok(Self { address, mtu_size, guid })
    }
}
