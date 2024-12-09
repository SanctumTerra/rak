use crate::BinaryStream;

use super::MAGIC;

pub struct ConnectionRequestOne {
    pub magic: [u8; 16],
    pub protocol: u8,
    pub mtu_size: u16
}

impl ConnectionRequestOne {
    pub const ID: u8 = 0x05;

    pub fn new(protocol: u8, mtu_size: u16) -> Self {
        Self { magic: MAGIC.to_vec().try_into().unwrap(), protocol, mtu_size }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID);
        stream.write((&self.magic).to_vec());
        stream.write_u8 (self.protocol);
        let udp_overhead = 28 as u16;
        let current_size = stream.binary.len() as u16;
        let padding_size = self.mtu_size - udp_overhead - current_size;
        stream.write(vec![0; padding_size as usize]);
        stream.binary
    }
}
