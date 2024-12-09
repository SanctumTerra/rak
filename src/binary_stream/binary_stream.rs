use super::Endianness;

pub struct BinaryStream {
    pub binary: Vec<u8>,
    pub offset: usize,
}

impl BinaryStream {
    pub fn new(binary: Option<Vec<u8>>, offset: Option<usize>) -> Self {
        Self { binary: binary.unwrap_or(Vec::new()), offset: offset.unwrap_or(0) }
    }

    pub fn read(&mut self, size: usize) -> Vec<u8> {
        if size > self.binary.len() {
            panic!("Size out of bounds");
        }

        if self.offset + size > self.binary.len() {
            panic!("\nOffset out of bounds\n");
        }

        let data = self.binary[self.offset..self.offset + size].to_vec();
        self.offset += size;
        data
    }

    pub fn write(&mut self, data: Vec<u8>) {
        self.binary.extend(data);
    }

    pub fn skip(&mut self, size: usize) {
        self.offset += size;
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub fn get_binary(&self) -> Vec<u8> {
        self.binary.clone()
    }

    pub fn set_binary(&mut self, binary: Vec<u8>) {
        self.binary = binary;
    }

    pub fn read_remaining(&mut self) -> Vec<u8> {
        self.binary[self.offset..self.binary.len()].to_vec()
    }

    pub fn cursor_at_start(&mut self) -> bool {
        self.offset == 0
    }

    pub fn cursor_at_end(&mut self) -> bool {
        self.offset == self.binary.len()
    }

    pub fn read_bytes(&mut self, size: usize) -> Vec<u8> {
        let data = self.read(size);
        data
    }

    pub fn write_bytes(&mut self, data: Vec<u8>) {
        self.write(data);
    }

    pub fn read_byte(&mut self) -> u8 {
        self.read(1)[0]
    }

    pub fn write_byte(&mut self, data: u8) {
        self.write(vec![data]);
    }

    pub fn read_u8(&mut self) -> u8 {
        self.read(1)[0]
    }

    pub fn write_u8(&mut self, data: u8) {
        self.write(vec![data]);
    }

   pub fn read_u16(&mut self, endianness: Option<Endianness>) -> u16 {
        let bytes = self.read(2);
        match endianness.unwrap_or(Endianness::Big) {
            Endianness::Big => u16::from_be_bytes(bytes.try_into().unwrap()),
            Endianness::Little => u16::from_le_bytes(bytes.try_into().unwrap()),
        }
    }

    pub fn write_u16(&mut self, data: u16, endianness: Option<Endianness>) {
        let bytes = match endianness.unwrap_or(Endianness::Big) {
            Endianness::Big => data.to_be_bytes(),
            Endianness::Little => data.to_le_bytes(),
        };
        self.write(bytes.to_vec());
    }

    pub fn read_u24(&mut self, endianness: Option<Endianness>) -> u32 {
        let endian = endianness.unwrap_or(Endianness::Big);
        let bytes = self.read(3);
        
        match endian {
            Endianness::Big => u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]),
            Endianness::Little => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]),
        }
    }

    pub fn write_u24(&mut self, data: u32, endianness: Option<Endianness>) {
        match endianness.unwrap_or(Endianness::Big) {
            Endianness::Big => self.write(data.to_be_bytes()[1..].to_vec()),
            Endianness::Little => self.write(data.to_le_bytes()[..3].to_vec()),
        }
    }

    pub fn read_u32(&mut self, endianness: Option<Endianness>) -> u32 {
        let data = self.read(4);
        match endianness.unwrap_or(Endianness::Big) {
            Endianness::Big => u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
            Endianness::Little => u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        }
    }

    pub fn write_u32(&mut self, data: u32, endianness: Option<Endianness>) {
        let bytes = match endianness {
            Some(Endianness::Big) => data.to_be_bytes(),
            Some(Endianness::Little) => data.to_le_bytes(),
            None => data.to_be_bytes(),
        };
        self.write(bytes.to_vec());
    }

    pub fn read_u64(&mut self, endianness: Option<Endianness>) -> u64 {
        let data = self.read(8);
        match endianness {
            Some(Endianness::Big) => u64::from_be_bytes(data.try_into().unwrap()),
            Some(Endianness::Little) => u64::from_le_bytes(data.try_into().unwrap()),
            None => u64::from_be_bytes(data.try_into().unwrap()),
        }
    }

    pub fn write_u64(&mut self, data: u64, endianness: Option<Endianness>) {
        let bytes = match endianness {
            Some(Endianness::Big) => data.to_be_bytes(),
            Some(Endianness::Little) => data.to_le_bytes(),
            None => data.to_be_bytes(),
        };
        self.write(bytes.to_vec());
    }

    pub fn read_bool(&mut self) -> bool {
        self.read_u8() != 0
    }

    pub fn write_bool(&mut self, data: bool) {
        self.write_u8(if data { 1 } else { 0 });
    }

    pub fn read_i8(&mut self) -> i8 {
        self.read_u8() as i8
    }

    pub fn write_i8(&mut self, data: i8) {
        self.write_u8(data as u8);
    }

    pub fn read_i16(&mut self, endianness: Option<Endianness>) -> i16 {
        self.read_u16(endianness) as i16
    }

    pub fn write_i16(&mut self, data: i16, endianness: Option<Endianness>) {
        self.write_u16(data as u16, endianness);
    }

    pub fn read_i24(&mut self, endianness: Option<Endianness>) -> i32 {
        self.read_u24(endianness) as i32
    }

    pub fn write_i24(&mut self, data: i32, endianness: Option<Endianness>) {
        self.write_u24(data as u32, endianness);
    }

    pub fn read_i32(&mut self, endianness: Option<Endianness>) -> i32 {
        let data = self.read(4);
        match endianness {
            Some(Endianness::Big) => i32::from_be_bytes(data.try_into().unwrap()),
            Some(Endianness::Little) => i32::from_le_bytes(data.try_into().unwrap()),
            None => i32::from_be_bytes(data.try_into().unwrap()),
        }
    }

    pub fn write_i32(&mut self, data: i32, endianness: Option<Endianness>) {
        let bytes = match endianness {
            Some(Endianness::Big) => data.to_be_bytes(),
            Some(Endianness::Little) => data.to_le_bytes(),
            None => data.to_be_bytes(),
        };
        self.write(bytes.to_vec());
    }

    pub fn read_i64(&mut self, endianness: Option<Endianness>) -> i64 {
        let bytes = self.read(8);
        match endianness {
            Some(Endianness::Big) => i64::from_be_bytes(bytes.try_into().unwrap()),
            Some(Endianness::Little) => i64::from_le_bytes(bytes.try_into().unwrap()),
            None => i64::from_be_bytes(bytes.try_into().unwrap()),
        }
    }

    pub fn write_i64(&mut self, data: i64, endianness: Option<Endianness>) {
        let bytes = match endianness {
            Some(Endianness::Big) => data.to_be_bytes(),
            Some(Endianness::Little) => data.to_le_bytes(),
            None => data.to_be_bytes(),
        };
        self.write(bytes.to_vec());
    }

    pub fn read_long(&mut self, endianness: Option<Endianness>) -> i64 {
        self.read_i64(endianness)
    }

    pub fn write_long(&mut self, data: i64, endianness: Option<Endianness>) {
        self.write_i64(data, endianness);
    }

    pub fn read_short(&mut self, endianness: Option<Endianness>) -> u16 {
        self.read_u16(endianness)
    }

    pub fn write_short(&mut self, data: u16, endianness: Option<Endianness>) {
        self.write_u16(data, endianness);
    }

    pub fn read_string16(&mut self, endianness: Option<Endianness>) -> Result<String, String> {
        let length = self.read_u16(endianness);
        let data = self.read(length as usize);
        match String::from_utf8(data) {
            Ok(string) => Ok(string),
            Err(e) => Err(format!("Invalid UTF-8 sequence: {}", e))
        }
    }

    pub fn write_string16(&mut self, data: &str, endianness: Option<Endianness>) {
        let bytes = data.as_bytes();
        self.write_u16(bytes.len() as u16, endianness);
        self.write(bytes.to_vec());
    }
}
