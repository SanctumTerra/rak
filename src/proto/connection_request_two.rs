use crate::BinaryStream;

use super::{Address, MAGIC};

pub struct ConnectionRequestTwo {
    pub magic: [u8; 16],
    pub address: Address,
    pub mtu_size: u16,
    pub guid: i64
}

impl ConnectionRequestTwo {
    pub const ID: u8 = 0x07;

    pub fn new(address: Address, mtu_size: u16, guid: i64) -> Self {
        Self { magic: MAGIC.to_vec().try_into().unwrap(), address, mtu_size, guid }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        stream.write((&self.magic).to_vec());
        self.address.write(&mut stream);
        stream.write_u16(self.mtu_size, None);
        stream.write_long(self.guid, None);
        stream.binary
    }

    pub fn deserialize(data: Vec<u8>) -> Result<Self, String> {
        let mut stream = BinaryStream::new(Some(data), None);
        stream.skip(1);
        let magic = stream.read(16).try_into().unwrap();
        let address = Address::read(&mut stream);
        let mtu_size = stream.read_u16(None);
        let guid = stream.read_long(None);
        Ok(Self { magic, address, mtu_size, guid })
    }
}
