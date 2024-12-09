#[deny(overflowing_literals)]

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use chrono::Utc;

use crate::proto::{ 
    Ack, Address, ConnectedPing, ConnectedPong, ConnectionRequest, ConnectionRequestAccepted, Frame, FrameSet, Nack, NewIncommingConnection, Priority, Reliability
};
use crate::socket::Socket;
use crate::binary_stream::BinaryStream;

use super::Event;

pub struct Framer {
    pub socket: Arc<Socket>,
    pub event_sender: Sender<Event>,
    pub mtu_size: u16,
    pub guid: i64,
    pub last_input_sequence: i32,
    pub received_frame_sequences: HashSet<u32>,
    pub lost_frame_sequences: HashSet<u32>,
    pub input_highest_sequence_index: [u32; 64],
    pub input_order_index: [u32; 64],
    pub input_ordering_queue: HashMap<u32, HashMap<u32, Frame>>,
    pub fragments_queue: HashMap<u16, HashMap<u32, Frame>>,

    pub output_sequence_index: [u32; 32],
    pub output_order_index: [u32; 32],
    pub output_frame_queue: FrameSet,
    pub output_frames: HashSet<Frame>,
    pub output_backup: HashMap<u32, Vec<Frame>>,
    pub output_sequence: u32,
    pub output_split_index: u32,
    pub output_reliable_index: u32,
}


impl Framer {
    pub fn new(socket: Arc<Socket>, mtu_size: u16, guid: i64, event_sender: Sender<Event>) -> Self {
        Self { 
            socket,
            event_sender,
            mtu_size, 
            guid,
            last_input_sequence: -1,
            received_frame_sequences: HashSet::new(),
            lost_frame_sequences: HashSet::new(),
            input_highest_sequence_index: [0; 64],
            input_order_index: [0; 64],
            input_ordering_queue: HashMap::new(),
            fragments_queue: HashMap::new(),

            output_sequence_index: [0; 32],
            output_order_index: [0; 32],
            output_frame_queue: FrameSet::new(0, Vec::new()),
            output_frames: HashSet::new(),
            output_backup: HashMap::new(),
            output_sequence: 0,
            output_split_index: 0,
            output_reliable_index: 0,
        }
    }

    pub fn handle_packet(&mut self, frame: &Frame) {
        if frame.payload.is_empty() {
            return;
        }
        let packet_id = frame.payload[0];
        
        match packet_id {
            ConnectedPing::ID => {
                self.emit_event("connected_ping", frame.payload.to_vec());
                let packet = ConnectedPing::deserialize(&frame.payload);
                let pong = ConnectedPong::new(packet.timestamp, Utc::now().timestamp());
                let mut framed = self.pls_frame(pong.serialize());
                self.send_frame(&mut framed, Some(Priority::Immediate));
            }
            ConnectionRequestAccepted::ID => {
                self.emit_event("connection_request_accepted", frame.payload.to_vec());
                let packet = ConnectionRequestAccepted::deserialize(&frame.payload).unwrap();
                let server_address = Address::new(4, self.socket.server_address.clone(), self.socket.server_port);
                let internal_addresses = core::array::from_fn(|_| Address::new(4, "127.0.0.1".to_string(), self.socket.get_address().port()));
                let response = NewIncommingConnection::new(
                    server_address, 
                    internal_addresses, 
                    Utc::now().timestamp(), 
                    packet.server_send_time
                );
                let mut framed = self.pls_frame(response.serialize());
                self.send_frame(&mut framed, Some(Priority::Immediate));
            }
            Ack::ID => {
                let ack = Ack::deserialize(&frame.payload);
                for sequence in ack.sequences {
                    self.output_backup.remove(&sequence);
                }
            }
            Nack::ID => {
                let nack = Nack::deserialize(&frame.payload);
                for seq in nack.sequences {
                    if self.output_backup.contains_key(&seq) {
                        let frames = self.output_backup.remove(&seq).unwrap();
                        for frame in frames {
                            let mut mframe = frame.clone();
                            self.send_frame(&mut mframe, Some(Priority::Immediate));
                        }
                        self.output_backup.remove(&seq);
                    }
                }
            }
            254 => {
                self.emit_event("encapsulated", frame.payload.to_vec());
            }
            21 => {
                self.emit_event("disconnect", frame.payload.to_vec());
            }
            _ => {
                self.emit_event("unknown_packet", frame.payload.to_vec());
                // println!("Received unknown packet: {:?}", packet_id);
            }
        }
    }

    pub fn emit_event(&self, name: &str, data: Vec<u8>) {
        let event = Event {
            name: name.to_string(),
            data,
        };
        self.event_sender.send(event).unwrap();
    }

    pub fn tick(&mut self) {
        let sequences: Vec<u32> = self.received_frame_sequences.clone()
            .into_iter()
            .collect();
        
        if !sequences.is_empty() {
            self.received_frame_sequences.clear();
            
            let ack = Ack::new(sequences);
            let packet = ack.serialize();
            let _ = self.socket.send(packet);
        }

        let nack_sequences: Vec<u32> = self.lost_frame_sequences.clone()
            .into_iter()
            .collect();
        
        if !nack_sequences.is_empty() {
            self.lost_frame_sequences.clear();
            
            let nack = Nack::new(nack_sequences);
            let packet = nack.serialize();
            let _ = self.socket.send(packet);
        }

        self.send_queue(self.output_frames.len());
    }

    pub fn on_frameset(&mut self, frameset: &FrameSet) {
        if self.received_frame_sequences.contains(&frameset.sequence) {
            return;
        }
        self.lost_frame_sequences.remove(&frameset.sequence);
        let sequence = frameset.sequence as i32;
        if sequence < self.last_input_sequence || sequence == self.last_input_sequence {
            return;
        }
        self.received_frame_sequences.insert(frameset.sequence);
        let diff = sequence - self.last_input_sequence;

        if diff != 1 {
            for index in (self.last_input_sequence + 1)..sequence {
                if !self.received_frame_sequences.contains(&(index as u32)) {
                    self.lost_frame_sequences.insert(index as u32);
                }
            }
        }
        self.last_input_sequence = sequence;

        for frame in &frameset.frames {
            self.handle_frame(frame);
        }
    }

    pub fn handle_frame(&mut self, frame: &Frame) {
        if frame.is_split() {
            self.handle_split_frame(frame);
        } else if frame.reliability.is_sequenced() {
            self.handle_sequenced_frame(frame);
        } else if frame.reliability.is_ordered() {
            self.handle_ordered_frame(frame);
        } else {
            self.handle_packet(frame);
        }
    }

    pub fn handle_split_frame(&mut self, frame: &Frame) {
        let split_id = frame.split_id.unwrap();
        if !self.fragments_queue.contains_key(&split_id) {
            self.fragments_queue.insert(split_id, HashMap::new());
        }

        let fragment = self.fragments_queue.get_mut(&split_id).unwrap();
        fragment.insert(frame.split_frame_index.unwrap(), frame.clone());

        if fragment.len() == frame.split_size.unwrap() as usize {
            let mut missing_fragments = false;
            for index in 0..frame.split_size.unwrap() {
                if !fragment.contains_key(&index) {
                    missing_fragments = true;
                    break;
                }
            }

            if missing_fragments {
                println!("Missing fragments for split packet {}", split_id);
                return;
            }
            
            let mut stream = BinaryStream::new(None, None);
            
            let first_reliable_index = fragment.get(&0)
                .map(|f| f.reliable_frame_index)
                .flatten();
            
            for index in 0..fragment.len() {
                if let Some(frame_) = fragment.get(&(index as u32)) {
                    stream.write_bytes(frame_.payload.clone());
                }
            }
            let mut reassembled_frame = Frame::new();
            reassembled_frame.reliability = frame.reliability.clone();
            reassembled_frame.reliable_frame_index = first_reliable_index;
            reassembled_frame.sequence_frame_index = frame.sequence_frame_index;
            reassembled_frame.ordered_frame_index = frame.ordered_frame_index;
            reassembled_frame.order_channel = frame.order_channel;
            reassembled_frame.payload = stream.binary;

            self.fragments_queue.remove(&split_id);
            self.handle_frame(&reassembled_frame);
        }
    }

    pub fn handle_sequenced_frame(&mut self, frame: &Frame) {
        let current_highest_sequence = self.input_highest_sequence_index[frame.order_channel.unwrap() as usize];

        if frame.sequence_frame_index.unwrap() > current_highest_sequence {
            self.input_highest_sequence_index[frame.order_channel.unwrap() as usize] = frame.sequence_frame_index.unwrap();
            self.handle_packet(frame);
        }
    }

    pub fn handle_ordered_frame(&mut self, frame: &Frame) {
        let channel = frame.order_channel.unwrap() as u32;
        let expected_order_index = self.input_order_index[frame.order_channel.unwrap() as usize];
        
        if !self.input_ordering_queue.contains_key(&channel) {
            self.input_ordering_queue.insert(channel, HashMap::new());
        }
        
        if frame.ordered_frame_index.unwrap() == expected_order_index {
            self.handle_packet(frame);
            self.input_order_index[frame.order_channel.unwrap() as usize] += 1;
            
            let mut next_order_index = expected_order_index + 1;
            let mut iterations = 0;
            const MAX_ITERATIONS: u32 = 1000;
            
            loop {
                iterations += 1;
                if iterations > MAX_ITERATIONS {
                    println!("Warning: Maximum iteration limit reached in handle_ordered_frame");
                    break;
                }

                let frame_to_handle = {
                    let out_of_order_queue = self.input_ordering_queue.get_mut(&channel).unwrap();
                    if let Some(next_frame) = out_of_order_queue.remove(&next_order_index) {
                        Some(next_frame)
                    } else {
                        None
                    }
                };
                
                match frame_to_handle {
                    Some(next_frame) => {
                        self.handle_packet(&next_frame);
                        self.input_order_index[frame.order_channel.unwrap() as usize] += 1;
                        next_order_index += 1;
                    }
                    None => break,
                }
            }
        } else if frame.ordered_frame_index.unwrap() > expected_order_index {
            let out_of_order_queue = self.input_ordering_queue.get_mut(&channel).unwrap();
            out_of_order_queue.insert(frame.ordered_frame_index.unwrap(), frame.clone());
        }
    }


    pub fn send_connect(&mut self) {
        let timestamp = Utc::now().timestamp();

        let packet = ConnectionRequest::new(
            self.guid, 
            timestamp,
            false
        );
        let mut frame = self.pls_frame(packet.serialize());
        self.send_frame(&mut frame, Some(Priority::Immediate));
    }

    pub fn pls_frame(&mut self, payload: Vec<u8>) -> Frame {
        let mut frame = Frame::new();
        frame.reliability = Reliability::ReliableOrdered;
        frame.order_channel = Some(0);
        frame.payload = payload;
        frame
    }

    pub fn send_frame(&mut self, frame: &mut Frame, priority: Option<Priority>) {
        let priority = priority.unwrap_or(Priority::Normal);
        
        if frame.reliability == Reliability::ReliableOrdered {
            frame.order_channel = Some(0);
            if frame.reliable_frame_index.is_none() {
                frame.reliable_frame_index = Some(self.output_reliable_index);
                self.output_reliable_index += 1;
            }
        }
        
        let order_channel = frame.order_channel.unwrap_or(0) as usize;

        if frame.reliability.is_sequenced() {
            frame.ordered_frame_index = Some(self.output_order_index[order_channel]);
            frame.sequence_frame_index = Some(self.output_sequence_index[order_channel]);
            self.output_sequence_index[order_channel] += 1;
        } else if frame.reliability.is_ordered() {
            let current_order_index = self.output_order_index[order_channel];
            frame.ordered_frame_index = Some(current_order_index);
            self.output_order_index[order_channel] = current_order_index + 1;
            self.output_sequence_index[order_channel] = 0;
        }

        let max_mtu = self.mtu_size - 36;

        if frame.payload.len() > max_mtu.into() {
            self.output_split_index += 1;
            let split_id = self.output_split_index % 65_536;
            let split_size = (frame.payload.len() as f64 / max_mtu as f64).ceil() as u32;
            let payload_len = frame.payload.len();

            let initial_reliable_index = self.output_reliable_index;
            self.output_reliable_index += 1;

            for split_index in 0..split_size {
                let start = (split_index as usize) * max_mtu as usize;
                let end = (start + max_mtu as usize).min(payload_len);    

                let mut split_frame = Frame::new();
                split_frame.sequence_frame_index = frame.sequence_frame_index;
                split_frame.ordered_frame_index = frame.ordered_frame_index;
                split_frame.order_channel = frame.order_channel;
                split_frame.split_id = Some(split_id as u16);
                split_frame.split_frame_index = Some(split_index);
                split_frame.split_size = Some(split_size);
                split_frame.reliability = frame.reliability.clone();
                split_frame.payload = frame.payload[start..end].to_vec();
                split_frame.reliable_frame_index = if split_index == 0 {
                    Some(initial_reliable_index)
                } else {
                    self.output_reliable_index += 1;
                    Some(self.output_reliable_index)
                };

                self.queue_frame(&split_frame, Some(priority.clone()));
            }
        } else {
            if frame.reliability.is_reliable() {
                frame.reliable_frame_index = Some(self.output_reliable_index);
                self.output_reliable_index += 1;
            }
            self.queue_frame(&frame, Some(priority));
        }
    }

    pub fn queue_frame(&mut self, frame: &Frame, priority: Option<Priority>) {
        let priority = priority.unwrap_or(Priority::Normal);
        let mut length = 4;

        for qframe in self.output_frames.iter() {
            length += qframe.get_size();
        }

        if length + frame.get_size() > (self.mtu_size - 36) as usize {
            self.send_queue(self.output_frames.len());
        }

        self.output_frames.insert(frame.clone());

        if priority == Priority::Immediate {
            self.send_queue(1);
        }
    }

    pub fn send_queue(&mut self, size: usize) {
        if self.output_frames.len() == 0 { return; }
        
        let frames: Vec<Frame> = self.output_frames.iter()
            .take(size)
            .cloned()
            .collect();
        
        self.output_sequence += 1;
        let frameset = FrameSet::new(self.output_sequence, frames);
        self.output_backup.insert(self.output_sequence, frameset.frames.clone());
        
        for frame in &frameset.frames {
            self.output_frames.remove(frame);
        }
        
        self.socket.send(frameset.serialize()).unwrap();
    }
}
