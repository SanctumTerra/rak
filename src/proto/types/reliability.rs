#[derive(Debug, Clone, Hash, Eq, PartialEq, Copy)]
pub enum Reliability {
    Unreliable = 0,
    UnreliableSequenced = 1,
    Reliable = 2,
    ReliableOrdered = 3,
    ReliableSequenced = 4,
    UnreliableWithAckReceipt = 5,
    ReliableWithAckReceipt = 6,
    ReliableOrderedWithAckReceipt = 7
}

impl Reliability {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Unreliable,
            1 => Self::UnreliableSequenced,
            2 => Self::Reliable,
            3 => Self::ReliableOrdered,
            4 => Self::ReliableSequenced,
            5 => Self::UnreliableWithAckReceipt,
            6 => Self::ReliableWithAckReceipt,
            7 => Self::ReliableOrderedWithAckReceipt,
            _ => Self::Unreliable
        }
    }

    pub fn is_reliable(&self) -> bool {
        matches!(self, 
            Self::Reliable | 
            Self::ReliableOrdered | 
            Self::ReliableSequenced |
            Self::ReliableWithAckReceipt |
            Self::ReliableOrderedWithAckReceipt
        )
    }

    pub fn is_sequenced(&self) -> bool {
        matches!(self, Self::ReliableSequenced | Self::UnreliableSequenced)
    }

    pub fn is_ordered(&self) -> bool {
        matches!(self, Self::ReliableOrdered | Self::ReliableOrderedWithAckReceipt)
    }
}