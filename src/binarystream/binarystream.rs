use std::error::Error;

use super::Endianness;

#[derive(Debug)]
pub enum BinaryStreamError {
    OutOfBounds,
    InvalidLength,
}

impl std::fmt::Display for BinaryStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for BinaryStreamError {}

#[derive(Debug)]
pub struct BinaryStream {
    pub binary: Vec<u8>,
    offset: u32
}

impl BinaryStream {
    pub fn new(binary: Option<Vec<u8>>, offset: Option<u32>) -> Self {
        let bin = match binary {
            Some(buffer) => buffer,
            None => vec![],
          };
      
          let offset = match offset {
            Some(offset) => offset,
            None => 0,
          };
      
        BinaryStream {
            binary: bin,
            offset,
        }
    }

    pub fn from(binary: Vec<u8>, offset: Option<u32>) -> Self {
        let offset = match offset {
            Some(offset) => offset,
            None => 0,
        };
    
        BinaryStream {
          binary: binary,
          offset,
        }
    }

    pub fn read(&mut self, length: u32) -> Result<Vec<u8>, BinaryStreamError> {
        if length > self.binary.len() as u32 {
            return Err(BinaryStreamError::InvalidLength);
        }
    
        if self.offset + length > self.binary.len() as u32 {
            return Err(BinaryStreamError::OutOfBounds);
        }
    
        let start = self.offset as usize;
        let end = (self.offset + length) as usize;
        self.offset += length;
    
        Ok(self.binary[start..end].to_vec())
    }

    pub fn write(&mut self, data: Vec<u8>) {
        self.binary.extend(data);
    }

    pub fn read_remaining(&mut self) -> Vec<u8> {
        let start = self.offset as usize;
        let end = self.binary.len();
        self.offset = end as u32;
    
        self.binary[start..end].to_vec()
    }

    pub fn skip(&mut self, length: u32) {
        self.offset += length;
    }

    pub fn cursor_at_end(&self) -> bool {
        self.offset == self.binary.len() as u32
    }
    
    pub fn cursor_at_start(&self) -> bool {
        self.offset == 0
    }

    pub fn get_offset(&self) -> u32 {
        self.offset
    }

    pub fn read_bytes(&mut self, length: u32) -> Result<Vec<u8>, BinaryStreamError> {
        Ok(self.read(length)?)
    }

    pub fn write_bytes(&mut self, data: Vec<u8>) {
        self.write(data);
    }

    pub fn read_u8(&mut self) -> Result<u8, BinaryStreamError> {
        let bytes = self.read(1)?;
        Ok(bytes[0])
    }

    pub fn write_u8(&mut self, value: u8) {
        self.write(vec![value]);
    }

    pub fn read_magic(&mut self) -> Result<Vec<u8>, BinaryStreamError> {
        let magic = self.read(16)?;
        Ok(magic)
    }

    pub fn write_magic(&mut self, magic: Vec<u8>) {
        self.write(magic);
    }

    pub fn read_u16(&mut self, endianness: Option<Endianness>) -> Result<u16, BinaryStreamError> {
        let bytes = self.read(2)?;
        Ok(match endianness.unwrap_or(Endianness::Big) {
            Endianness::Big => u16::from_be_bytes(bytes.try_into().unwrap()),
            Endianness::Little => u16::from_le_bytes(bytes.try_into().unwrap()),
        })
    }

    pub fn write_u16(&mut self, value: u16, endianness: Option<Endianness>) {
        let bytes = match endianness.unwrap_or(Endianness::Big) {
            Endianness::Big => value.to_be_bytes(),
            Endianness::Little => value.to_le_bytes(),
        };
        self.write(bytes.to_vec());
    }

    pub fn read_u64(&mut self, endianness: Option<Endianness>) -> Result<u64, BinaryStreamError> {
        let bytes = self.read(8)?;
        let bytes: [u8; 8] = bytes.try_into()
            .map_err(|_| BinaryStreamError::InvalidLength)?;
            
        Ok(match endianness.unwrap_or(Endianness::Little) {
            Endianness::Big => u64::from_be_bytes(bytes),
            Endianness::Little => u64::from_le_bytes(bytes),
        })
    }

    pub fn write_u64(&mut self, value: u64, endianness: Option<Endianness>) {
        let bytes = match endianness.unwrap_or(Endianness::Little) {
            Endianness::Big => value.to_be_bytes(),
            Endianness::Little => value.to_le_bytes(),
        };
        self.write(bytes.to_vec());
    }

    pub fn read_u32(&mut self, endianness: Option<Endianness>) -> Result<u32, BinaryStreamError> {
        let endian = match endianness {
            Some(endian) => endian,
            None => Endianness::Big,
        };
      
        let bytes = match self.read(4) {
            Ok(bytes) => bytes,
            Err(err) => return Err(err)
        };
          
        match endian {
            Endianness::Big => Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
            Endianness::Little => Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
        }
    }

    pub fn write_u32(&mut self, value: u32, endianness: Option<Endianness>) {
        let endian = match endianness {
            Some(endian) => endian,
            None => Endianness::Big,
        };
        match endian {
            Endianness::Big => self.write(value.to_be_bytes().to_vec()),
            Endianness::Little => self.write(value.to_le_bytes().to_vec()),
        }
    }

    pub fn read_bool(&mut self) -> Result<bool, BinaryStreamError> {
        let value = self.read_u8()?;
        Ok(value != 0)
    }

    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(if value { 1 } else { 0 });
    }

    pub fn read_u24(&mut self, endianness: Option<Endianness>) -> Result<u32, BinaryStreamError> {
        let endian = match endianness {
            Some(endian) => endian,
            None => Endianness::Big,
        };
          
        let bytes = match self.read(3) {
            Ok(bytes) => bytes,
            Err(err) => return Err(err)
        };
      
        match endian {
            Endianness::Big => Ok(u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]])),
            Endianness::Little => Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0])),
        }
    }

    pub fn write_u24(&mut self, value: u32, endianness: Option<Endianness>) {
        let endian = match endianness {
            Some(endian) => endian,
            None => Endianness::Big,
        };
      
        match endian {
            Endianness::Big => self.write(value.to_be_bytes()[1..].to_vec()),
            Endianness::Little => self.write(value.to_le_bytes()[..3].to_vec()),
        }    
    }

    pub fn read_i16(&mut self, endianness: Option<Endianness>) -> Result<i16, BinaryStreamError> {
        let endian = match endianness {
            Some(endian) => endian,
            None => Endianness::Big,
        };
      
        let bytes = match self.read(2) {
            Ok(bytes) => bytes,
            Err(err) => return Err(err)
        };
      
        match endian {
            Endianness::Big => Ok(i16::from_be_bytes([bytes[0], bytes[1]])),
            Endianness::Little => Ok(i16::from_le_bytes([bytes[0], bytes[1]])),
        }
    }

    pub fn write_i16(&mut self, value: i16, endianness: Option<Endianness>) {
        let endian = match endianness {
            Some(endian) => endian,
            None => Endianness::Big,
        };

        match endian {
            Endianness::Big => self.write(value.to_be_bytes().to_vec()),
            Endianness::Little => self.write(value.to_le_bytes().to_vec()),
        }
    }

    pub fn read_i64(&mut self, endianness: Option<Endianness>) -> Result<i64, BinaryStreamError> {
        let bytes = self.read(8)?;
        let bytes: [u8; 8] = bytes.try_into()
            .map_err(|_| BinaryStreamError::InvalidLength)?;
        Ok(match endianness.unwrap_or(Endianness::Big) {
            Endianness::Big => i64::from_be_bytes(bytes),
            Endianness::Little => i64::from_le_bytes(bytes),
        })
    }

    pub fn write_i64(&mut self, value: i64, endianness: Option<Endianness>) {
        let endian = match endianness {
            Some(endian) => endian,
            None => Endianness::Big,
        };
      
        match endian {
            Endianness::Big => self.write(value.to_be_bytes().to_vec()),
            Endianness::Little => self.write(value.to_le_bytes().to_vec()),
        }
    }

    pub fn read_long(&mut self, endianness: Option<Endianness>) -> Result<i64, BinaryStreamError> {
        Ok(self.read_i64(Some(endianness.unwrap_or(Endianness::Big)))?)
    }

    pub fn write_long(&mut self, value: i64, endianness: Option<Endianness>) {
        self.write_i64(value, Some(endianness.unwrap_or(Endianness::Big)));
    }

    pub fn read_short(&mut self, endianness: Option<Endianness>) -> Result<u16, BinaryStreamError> {
        let bytes = self.read(2)?;

        match endianness.unwrap_or(Endianness::Big) {
            Endianness::Big => Ok((bytes[0] as u16) << 8 | bytes[1] as u16),
            Endianness::Little => Ok((bytes[1] as u16) << 8 | bytes[0] as u16),
        }
    }

    pub fn write_short(&mut self, value: u16, endianness: Option<Endianness>) {
        match endianness.unwrap_or(Endianness::Big) {
            Endianness::Big => {
                self.write(vec![(value >> 8) as u8, (value & 0xFF) as u8]);
            },
            Endianness::Little => {
                self.write(vec![(value & 0xFF) as u8, (value >> 8) as u8]);
            }
        }
    }

    pub fn read_byte(&mut self) -> Result<u8, BinaryStreamError> {
        let byte = self.read(1)?;
        Ok(byte[0])
    }

    pub fn write_byte(&mut self, value: u8) {
        self.write(vec![value]);
    }
}
