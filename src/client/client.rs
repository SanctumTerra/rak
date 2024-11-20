use chrono::Utc;
use rand::Rng;

use crate::{
    packets::{
        open_connection_request_one::OpenConnectionRequestOne, packet::Packet, packet_types::PacketType, Ack, Address, ConnectionRequest, Frame, FrameSet, OpenConnectionReplyOne, OpenConnectionReplyTwo, OpenConnectionRequestTwo, Priority, Reliability, UnconnectedPing
    },
    socket::{Socket, SocketError},
};

use super::Framer;

pub struct Client {
    socket: Socket,
    address: String,
    pub mtu_size: u16,
    framer: Framer,
    port: u16,
    guid: i64,
    step: u32,
    debug: bool,
}

impl Client {
    pub fn new(address: Option<String>, port: Option<u16>, mtu_size: Option<u16>, debug: Option<bool>) -> Result<Self, SocketError> {
        let socket = Socket::new(None, None)?;
        let address = address.unwrap_or("127.0.0.1".to_string());
        let port = port.unwrap_or(socket.get_local_port());
        let guid = rand::thread_rng().gen_range(0..i64::MAX);
        let mtu_size = mtu_size.unwrap_or(1492);
        let debug = debug.unwrap_or(false);
        
        let framer = Framer::new(socket.clone(), mtu_size, address.clone(), port, debug);
        Ok(Self { socket, address, port, guid, step: 0, mtu_size, framer, debug })
    }

    pub fn send(&self, buffer: &[u8]) -> Result<usize, SocketError> {
        self.socket.send(buffer, &format!("{}:{}", self.address, self.port))
    }

    pub fn tick(&mut self) -> Result<(), SocketError> {
        let _ = self.framer.tick();
        Ok(())
    }

    pub fn ping(&self) -> Result<(), SocketError> {
        let timestamp = Utc::now().timestamp();
        let packet = UnconnectedPing::new(timestamp, self.guid);
        let serialized = packet.serialize().unwrap();
        self.send(&serialized)?;
        Ok(())
    }

    pub fn frame_and_send(&mut self, buffer: &[u8]) -> Result<(), SocketError> {
        let mut frame = self.pls_frame(buffer)?;
        self.framer.send_frame(&mut frame, Priority::Immediate).unwrap();
        Ok(())
    }

    pub fn get_timestamp(&self) -> i64 {
        Utc::now().timestamp()
    }

    pub fn connect(&mut self) -> Result<(), SocketError> {
        let mut mtu_size = self.mtu_size;
        if self.step == 1 {
            mtu_size = 1200;
        }
        if self.step == 2 {
            mtu_size = 576;
        }

        let packet = OpenConnectionRequestOne {
            protocol_version: 11,
            mtu_size: mtu_size as usize,
        };


        let serialized = packet.serialize().unwrap();
        self.send(&serialized)?;
        Ok(())
    }

    pub fn receive(&mut self) -> Result<Vec<u8>, SocketError> {
        let mut buffer = [0; 1500];
        let (size, _) = self.socket.receive(&mut buffer).unwrap();
        
        if size > 0 {
            let _ = self.handle_packet(&buffer[..size]);
        }
        Ok(buffer[..size].to_vec())
    }

    fn pls_frame(&mut self, buffer: &[u8]) -> Result<Frame, SocketError> {
        let frame = Frame {
            reliability: Reliability::ReliableOrdered,
            reliable_frame_index: Some(1),
            sequence_frame_index: None,
            ordered_frame_index: Some(0),
            order_channel: Some(0),
            split_frame_index: None,
            split_size: None,
            split_id: None,
            payload: buffer.to_vec()
        };
        Ok(frame)
    }

    fn send_frame(&mut self, buffer: &[u8]) -> Result<(), SocketError> {
        let mut frame = Frame {
            reliability: Reliability::ReliableOrdered,
            reliable_frame_index: Some(1),
            sequence_frame_index: None,
            ordered_frame_index: Some(0),
            order_channel: Some(0),
            split_frame_index: None,
            split_size: None,
            split_id: None,
            payload: buffer.to_vec()
        };
        self.framer.send_frame(&mut frame, Priority::Immediate).unwrap();
        Ok(())
    }

    pub fn handle_packet(&mut self, buffer: &[u8]) -> Result<(), SocketError> {
        if buffer.is_empty() {
            return Ok(());
        }
        let mut packet_type = PacketType::from(buffer[0]);
        
        if (packet_type.to_u8() & 0xf0) == 0x80 {
            packet_type = PacketType::FrameSet;
        }

        match packet_type {
            PacketType::UnconnectedPong => {
                self.framer.add_received_packet(buffer.to_vec());
            },
            PacketType::OpenConnectionReplyOne => {
                let packet = OpenConnectionReplyOne::deserialize(buffer).unwrap();
                let request_two = OpenConnectionRequestTwo {
                    address: Address::new(self.address.clone(), self.port, 4),
                    mtu_size: packet.mtu_size,
                    guid: self.guid
                };
                let serialized = request_two.serialize().unwrap();
                self.send(&serialized)?;
                if packet.mtu_size > 1500 {
                    self.connect()?;
                }
            },
            PacketType::OpenConnectionReplyTwo => {
                let packet = OpenConnectionReplyTwo::deserialize(buffer).unwrap();
                let conn_request = ConnectionRequest {
                    guid: self.guid,
                    timestamp: Utc::now().timestamp(),
                    security: false,
                };
                let serialized = conn_request.serialize().unwrap();
                self.send_frame(&serialized)?;
                if packet.mtu_size > 1500 {
                    self.connect()?;
                }
            },
            PacketType::FrameSet => {
                let _ = self.framer.on_frame_set(&FrameSet::deserialize(buffer).unwrap());
            },
            PacketType::Ack => {
                let frame = Frame {
                    reliability: Reliability::ReliableOrdered,
                    reliable_frame_index: Some(1),
                    sequence_frame_index: None,
                    ordered_frame_index: Some(0),
                    order_channel: Some(0),
                    split_frame_index: None,
                    split_size: None,
                    split_id: None,
                    payload: buffer.to_vec()
                };
                self.framer.process_packet(&frame).unwrap();
            },
            PacketType::Nack => {
                let frame = self.pls_frame(buffer)?;
                self.framer.process_packet(&frame).unwrap();
            },
            _ => {}
        }

        Ok(())
    }

    pub fn get_received_packets(&mut self) -> Result<Vec<Vec<u8>>, SocketError> {
        let mut buffer = [0; 1500];
        let (size, _) = self.socket.receive(&mut buffer)?;
        
        if size > 0 {
            let _ = self.handle_packet(&buffer[..size]);
        }

        Ok(self.framer.get_received_packets())
    }
}
