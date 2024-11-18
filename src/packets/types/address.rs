use super::DataType;
use crate::{binarystream::BinaryStream, packets::PacketError};
use std::net::{IpAddr, Ipv6Addr};

#[derive(Debug, Clone)]
pub struct Address {
    pub address: String,
    pub port: u16,
    pub version: u8
}

impl DataType for Address {
    fn read(stream: &mut BinaryStream) -> Self {
        let version = stream.read_u8().unwrap();
        if version == 4 {
            let address = format!(
                "{}.{}.{}.{}",
                !stream.read_byte().unwrap() & 0xff,
                !stream.read_byte().unwrap() & 0xff,
                !stream.read_byte().unwrap() & 0xff,
                !stream.read_byte().unwrap() & 0xff
            );
            let port = stream.read_short(None).unwrap();
            Self { address, port, version }
        } else if version == 6 {
            stream.read_u16(None).unwrap(); // AF_INET6
            let port = stream.read_u16(None).unwrap();
            stream.read_u32(None).unwrap(); // flow info
            
            let bytes = stream.read(16).unwrap();
            stream.read_u32(None).unwrap();
            
            let byte_array: [u8; 16] = bytes.try_into().expect("Incorrect length for IPv6 address");
            let ipv6 = Ipv6Addr::from(byte_array);
            let address = IpAddr::V6(ipv6).to_string();
            
            Self { address, port, version }
        } else {
            Self { address: "".to_string(), port: 0, version }
        }
    }

    fn write(&self, stream: &mut BinaryStream) -> Result<(), PacketError> {
        stream.write_u8(self.version);
        
        if self.version == 4 {
            let parts: Vec<&str> = self.address.split('.').collect();
            assert_eq!(parts.len(), 4, "Wrong number of parts in IPv4 IP, expected 4, got {}", parts.len());
            for part in parts {
                let b: u8 = part.parse().unwrap();
                stream.write_u8((!b) & 0xff);
            }
            stream.write_u16(self.port, None);

        } else if self.version == 6 {
            stream.write_u16(23, None);
            stream.write_u16(self.port, None);
            stream.write_u32(0, None);

            let parts: Vec<&str> = self.address.split(':').collect();
            for part in parts {
                let num = u16::from_str_radix(part, 16).expect("Invalid IPv6 part");
                stream.write_u16(num ^ 0xffff, None);
            }
            stream.write_u32(0, None);
        }
        Ok(())
    }
}

impl Address {
    pub fn new(address: String, port: u16, version: u8) -> Self {
        Self { address, port, version }
    }
}
