use crate::BinaryStream;

use super::{Address, MAGIC};

#[derive(Debug, Clone)]
pub struct ConnectionReplyTwo {
    pub magic: [u8; 16],
    pub guid: i64,
    pub address: Address,
    pub mtu_size: u16,
    pub encryption_enabled: bool
}

impl ConnectionReplyTwo {
    pub const ID: u8 = 0x08;

    pub fn new(guid: i64, address: Address, mtu_size: u16, encryption_enabled: bool) -> Self {
        Self { magic: MAGIC.to_vec().try_into().unwrap(), guid, address, mtu_size, encryption_enabled }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        stream.write((&self.magic).to_vec());
        stream.write_long(self.guid, None);
        self.address.write(&mut stream);
        stream.write_u16(self.mtu_size, None);
        stream.write_bool(self.encryption_enabled);
        stream.binary
    }

    pub fn deserialize(data: Vec<u8>) -> Result<Self, String> {
        let mut stream = BinaryStream::new(Some(data), None);
        stream.skip(1);
        let magic = stream.read(16).try_into().unwrap();
        let guid = stream.read_long(None);
        let address = Address::read(&mut stream);
        let mtu_size = stream.read_u16(None);
        let encryption_enabled = stream.read_bool();
        Ok(Self { magic, guid, address, mtu_size, encryption_enabled })
    }
}
