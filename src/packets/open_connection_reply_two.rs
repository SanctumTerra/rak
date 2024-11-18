use crate::binarystream::BinaryStream;

use super::{Address, DataType, PacketError, MAGIC};

pub struct OpenConnectionReplyTwo {
    pub guid: i64,
    pub address: Address,
    pub mtu_size: u16,
    pub encryption_enabled: bool,
    pub encryption_key: Option<Vec<u8>>,
}

impl OpenConnectionReplyTwo {
    pub const ID: u8 = 0x06;
    const MAGIC: [u8; 16] = MAGIC;

    pub fn deserialize(data: &[u8]) -> Result<Self, PacketError> {
        let mut stream = BinaryStream::new(Some(data.to_vec()), None);
        stream.read_u8()?;
        stream.read_magic()?;
        let guid = stream.read_long(None)?;
        let address = Address::read(&mut stream);
        let mtu_size = stream.read_u16(None)?;
        let encryption_enabled = stream.read_bool()?;
        let encryption_key = if encryption_enabled { Some(stream.read_bytes(128)?) } else { None };
        Ok(Self { guid, address, mtu_size, encryption_enabled, encryption_key })
    }

    pub fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        stream.write_magic(Self::MAGIC.to_vec());
        stream.write_long(self.guid, None);
        self.address.write(&mut stream)?;
        stream.write_u16(self.mtu_size, None);
        stream.write_bool(self.encryption_enabled);
        if let Some(encryption_key) = &self.encryption_key {
            stream.write_bytes(encryption_key.clone());
        }
        Ok(stream.binary)
    }
}

