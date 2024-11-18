use std::error::Error;
use std::fmt;
use crate::binarystream::binarystream::BinaryStreamError;

#[derive(Debug)]
pub enum PacketError {
    InvalidPacket,
    BinaryStreamError(BinaryStreamError),
    SerializationError,
}



impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PacketError::InvalidPacket => write!(f, "Invalid packet"),
            PacketError::BinaryStreamError(e) => write!(f, "Binary stream error: {}", e),
            PacketError::SerializationError => write!(f, "Serialization error"),
        }
    }
}

impl Error for PacketError {}

impl From<BinaryStreamError> for PacketError {
    fn from(error: BinaryStreamError) -> Self {
        PacketError::BinaryStreamError(error)
    }
}

pub const MAGIC: [u8; 16] = [
    0x00, 0xFF, 0xFF, 0x00,
    0xFE, 0xFE, 0xFE, 0xFE,
    0xFD, 0xFD, 0xFD, 0xFD,
    0x12, 0x34, 0x56, 0x78
];

pub trait Packet {
    const ID: u16;
    const MAGIC: [u8; 16];

    fn serialize(&self) -> Result<Vec<u8>, PacketError>;
    fn deserialize(buffer: &[u8]) -> Result<Self, PacketError> where Self: Sized;
}
