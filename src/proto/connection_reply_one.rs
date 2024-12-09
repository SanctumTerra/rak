use crate::BinaryStream;

#[derive(Debug, Clone)]
pub struct ConnectionReplyOne {
    pub magic: [u8; 16],
    pub guid: i64,
    pub security: bool,
    pub cookie: Option<u32>,
    pub mtu_size: u16
}

impl ConnectionReplyOne {
    pub const ID: u8 = 0x06;

    pub fn serialize() { 

    }

    pub fn deserialize(data: Vec<u8>) -> Result<Self, String> {
        let mut stream = BinaryStream::new(Some(data), None);
        stream.skip(1);
        let magic = stream.read(16).try_into().unwrap();
        let guid = stream.read_long(None);
        let security = stream.read_bool();
        let cookie = if security { Some(stream.read_u32(None)) } else { None };
        let mtu_size = stream.read_u16(None);
        Ok(Self { magic, guid, security, cookie, mtu_size })
    }
}