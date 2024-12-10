#![allow(dead_code)]

use chrono::Utc;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::proto::{ 
    Ack, Address, ConnectionReplyOne, ConnectionReplyTwo, ConnectionRequestOne, ConnectionRequestTwo, FrameSet, Nack, UnconnectedPing, UnconnectedPong
};
use crate::socket::Socket;
use crate::Priority;

use super::Framer;

#[derive(Clone, Debug)]
pub struct Event {
    pub name: String,
    pub data: Vec<u8>,
}

pub struct Client {
    pub socket: Arc<Socket>,
    pub guid: i64,
    pub mtu_size: u16,
    pub framer: Framer,
    pub tick_count: u32,
    pub event_sender: Sender<Event>,
    pub event_receiver: Receiver<Event>,
    pub connected: AtomicBool,
}

impl Client {
    pub fn new(host: String, port: u16) -> Self {
        let guid = 4124124124124;
        let mtu_size = 1492;
        let socket = Arc::new(Socket::new(host, port));
        let (event_sender, event_receiver) = channel();
        let framer = Framer::new(
            Arc::clone(&socket), 
            mtu_size, 
            guid, 
            event_sender.clone()
        );
        
        Self { 
            socket, 
            guid, 
            mtu_size, 
            framer,
            tick_count: 0,
            event_sender,
            event_receiver,
            connected: AtomicBool::new(false),
        }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        if self.tick_count > 0 { 
            self.mtu_size = 1200;
        }
        let timestamp = Utc::now().timestamp();
        let ping = UnconnectedPing::new(timestamp, self.guid);
        self.socket.send(ping.serialize()).unwrap();
        let request = ConnectionRequestOne::new(11, self.mtu_size);
        self.socket.send(request.serialize()).unwrap();
        Ok(())
    }

    pub fn ping(&mut self) {
        let timestamp = Utc::now().timestamp();
        let ping = UnconnectedPing::new(timestamp, self.guid);
        self.socket.send(ping.serialize()).unwrap();
    }

    pub fn receive(&mut self) -> Result<Vec<u8>, String> {
        let mut buffer = [0; 1492];
        match self.socket.receive(&mut buffer) {
            Ok(0) => Ok(vec![]),
            Ok(size) => {
                self.handle_packet(&buffer[..size]);
                Ok(buffer[..size].to_vec())
            }
            Err(e) => Err(e.to_string())
        }
    }

    pub fn tick(&mut self) {
        // let _ = self.receive();
        self.framer.tick();
    }

    pub fn send(&mut self, data: Vec<u8>) -> Result<(), String> {
        self.socket.send(data).unwrap();
        Ok(())
    }

    pub fn emit_event(&self, name: &str, data: Vec<u8>) {
        let event = Event {
            name: name.to_string(),
            data,
        };
        self.event_sender.send(event).unwrap();
    }

    pub fn frame_and_send(&mut self, data: Vec<u8>) {
        let mut frame = self.framer.pls_frame(data);
        self.framer.send_frame(&mut frame, Some(Priority::Immediate));
    }

    pub fn handle_packet(&mut self, binary: &[u8]) {
        let mut packet_id = binary[0];
        if (packet_id & 0xf0) == 0x80 {
            packet_id = FrameSet::ID;
        }

        match packet_id {
            UnconnectedPong::ID => {
                // let packet = UnconnectedPong::deserialize(binary.to_vec()).unwrap();
                self.emit_event("unconnected_pong", binary.to_vec());
            }
            ConnectionReplyOne::ID => {
                let packet = ConnectionReplyOne::deserialize(binary.to_vec()).unwrap();
                self.emit_event("connection_reply_one", binary.to_vec());
                let request = ConnectionRequestTwo::new(
                    Address::new(4, self.socket.server_address.clone(), self.socket.server_port), 
                    packet.mtu_size, 
                    self.guid
                );
                self.send(request.serialize()).unwrap();
                if packet.mtu_size > 1500 || packet.mtu_size < 400 {
                    self.connect().unwrap();
                } 
            }
            ConnectionReplyTwo::ID => {
                let _packet = ConnectionReplyTwo::deserialize(binary.to_vec()).unwrap();
                self.emit_event("connection_reply_two", binary.to_vec());
                self.connected.store(true, Ordering::SeqCst);
                self.framer.send_connect();
            }
            FrameSet::ID => {
                let packet = FrameSet::deserialize(&binary).unwrap();
                self.framer.on_frameset(&packet);
                self.emit_event("frame_set", binary.to_vec());
            }
            Ack::ID => {
                self.emit_event("ack", binary.to_vec());
                let frame = self.framer.pls_frame(binary.to_vec());
                self.framer.handle_packet(&frame);
            }
            Nack::ID => {
                self.emit_event("nack", binary.to_vec());
                let frame = self.framer.pls_frame(binary.to_vec());
                self.framer.handle_packet(&frame);
            }
            21 => {
                self.connected.store(false, Ordering::SeqCst);
                self.emit_event("disconnect", binary.to_vec());
            }
            _ => {
                println!("Received unknown packet: {}", packet_id);
                self.emit_event("unknown_packet", vec![packet_id]);
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
}

fn dump_packet(prefix: &str, data: &[u8]) {
    println!("{} packet [{} bytes]: {:02X?}", prefix, data.len(), data);
}
