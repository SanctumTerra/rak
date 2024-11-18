use crate::{binarystream::BinaryStream, packets::PacketError};

pub trait DataType: Sized {
    fn read(stream: &mut BinaryStream) -> Self;
    fn write(&self, stream: &mut BinaryStream) -> Result<(), PacketError>;
}