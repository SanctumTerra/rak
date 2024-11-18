use crate::binarystream::BinaryStream;

use super::{Packet, PacketError, MAGIC};

pub struct OpenConnectionReplyOne {
    pub server_guid: u64,
    pub security: bool,
    pub cookie: Option<u32>,
    pub mtu_size: u16,
}

impl Packet for OpenConnectionReplyOne {
    const ID: u16 = 0x06;
    const MAGIC: [u8; 16] = MAGIC;

    fn serialize(&self) -> Result<Vec<u8>, PacketError> {
        let mut stream = BinaryStream::new(None, None);
        stream.write_u8(Self::ID as u8);
        stream.write_magic(Self::MAGIC.to_vec());
        stream.write_u64(self.server_guid, None);
        stream.write_bool(self.security);
        if let Some(cookie) = self.cookie {
            stream.write_u32(cookie, None);
        }
        stream.write_u16(self.mtu_size, None);
        Ok(stream.binary)
    }   

    fn deserialize(buffer: &[u8]) -> Result<Self, PacketError> {
        let mut stream = BinaryStream::from(buffer.to_vec(), Some(0));
        stream.read_u8()?;
        stream.read_magic()?;
        let server_guid = stream.read_u64(None)?;
        let security = stream.read_bool()?;
        let cookie = if security {
            Some(stream.read_u32(None)?)
        } else {
            None
        };
        let mtu_size = stream.read_u16(None)?;
        Ok(Self {
            server_guid,
            security,
            cookie,
            mtu_size,
        })
    }
}
