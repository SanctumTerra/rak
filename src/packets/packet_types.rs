#[derive(Debug)]
pub enum PacketType {
    ConnectedPing,
    UnconnectedPing,
    ConnectedPong,
    OpenConnectionRequestOne,
    OpenConnectionReplyOne,
    OpenConnectionRequestTwo,
    OpenConnectionReplyTwo,
    FrameSet,
    ConnectionRequest,
    ConnectionRequestAccepted,
    UnconnectedPong,
    Ack,
    Nack,
    NewIncomingConnection,
    Unknown(u8),
}

impl From<u8> for PacketType {
    fn from(id: u8) -> Self {
        match id {
            0x00 => PacketType::ConnectedPing,
            0x01 => PacketType::UnconnectedPing,
            0x03 => PacketType::ConnectedPong,
            0x05 => PacketType::OpenConnectionRequestOne,
            0x06 => PacketType::OpenConnectionReplyOne,
            0x07 => PacketType::OpenConnectionRequestTwo,
            0x08 => PacketType::OpenConnectionReplyTwo,
            0x80 => PacketType::FrameSet,
            0x09 => PacketType::ConnectionRequest,
            0x10 => PacketType::ConnectionRequestAccepted,
            0x1c => PacketType::UnconnectedPong,
            0xc0 => PacketType::Ack,
            0xa0 => PacketType::Nack,
            0x13 => PacketType::NewIncomingConnection,
            id => PacketType::Unknown(id),
        }
    }
}

impl PacketType {
    pub fn to_u8(&self) -> u8 {
        match self {
            PacketType::ConnectedPing => 0x00,
            PacketType::UnconnectedPing => 0x01,
            PacketType::ConnectedPong => 0x03,
            PacketType::FrameSet => 0x80,
            PacketType::OpenConnectionReplyOne => 0x06,
            PacketType::OpenConnectionReplyTwo => 0x08,
            PacketType::OpenConnectionRequestOne => 0x05,
            PacketType::OpenConnectionRequestTwo => 0x07,
            PacketType::ConnectionRequest => 0x09,
            PacketType::ConnectionRequestAccepted => 0x10,
            PacketType::UnconnectedPong => 0x1c,
            PacketType::Ack => 0xc0,
            PacketType::Nack => 0xa0,
            PacketType::NewIncomingConnection => 0x13,
            PacketType::Unknown(id) => *id,
        }
    }
} 