use std::collections::{HashMap, HashSet};
use crate::binarystream::BinaryStream;
use crate::packets::{Ack, Address, ConnectedPing, ConnectedPong, ConnectionRequestAccepted, Frame, FrameSet, Nack, NewIncomingConnection, Packet, PacketType, Priority, Reliability};
use crate::socket::Socket;
use std::error::Error;
use std::fmt;
use chrono::Utc;

#[derive(Debug)]
pub enum FramerError {
    InvalidSequence,
}

impl fmt::Display for FramerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Framer error: {}", self)
    }
}

impl Error for FramerError {}

pub struct Framer {
    mtu_size: u16,
    socket: Socket,
    address: String,
    port: u16,
    debug: bool,
    last_input_sequence: i32,
    received_frame_sequences: HashSet<u32>,
    lost_frame_sequences: HashSet<u32>,
    input_highest_sequence_index: [u32; 64],
    input_order_index: [u32; 64],
    input_ordering_queue: HashMap<u32, HashMap<u32, Frame>>,
    fragments_queue: HashMap<u32, HashMap<u32, Frame>>,
    output_order_index: [u32; 64],
    output_sequence_index: [u32; 64],
    output_frame_queue: FrameSet,
    output_split_index: u32,
    output_reliable_index: u32,
    output_frames: HashSet<Frame>,
    output_backup: HashMap<u32, Vec<Frame>>,
    output_sequence: u32,
    received_packets: Vec<Vec<u8>>,
}

impl Framer {
    pub fn new(socket: Socket, mtu_size: u16, address: String, port: u16, debug: bool) -> Self {
        Self {
            mtu_size,
            socket,
            address,
            port,
            last_input_sequence: -1,
            received_frame_sequences: HashSet::new(),
            lost_frame_sequences: HashSet::new(),
            input_highest_sequence_index: [0; 64],
            input_order_index: [0; 64],
            input_ordering_queue: HashMap::new(),
            fragments_queue: HashMap::new(),
            output_order_index: [0; 64],
            output_sequence_index: [0; 64],
            output_frame_queue: FrameSet {
                sequence: 0,
                frames: Vec::new(),
            },
            output_split_index: 0,
            output_reliable_index: 0,
            output_frames: HashSet::new(),
            output_backup: HashMap::new(),
            output_sequence: 0,
            received_packets: Vec::new(),
            debug,
        }
    }

    pub fn on_frame_set(&mut self, frameset: &FrameSet) -> Result<(), FramerError> {
        if self.debug {
            println!("Received frameset sequence: {} (last: {})", 
                    frameset.sequence, self.last_input_sequence);
        }

        if self.received_frame_sequences.contains(&frameset.sequence) {
            if self.debug {
                println!("Duplicate frameset received with sequence: {}", frameset.sequence);
            }
            return Ok(());
        }

        self.received_frame_sequences.insert(frameset.sequence);
        self.lost_frame_sequences.remove(&frameset.sequence);
        
        let frame_seq_i32 = frameset.sequence as i32;
        
        if frame_seq_i32 <= self.last_input_sequence {
            if self.debug {
                println!("Out of order frameset: {} (expected > {})", 
                        frame_seq_i32, self.last_input_sequence);
            }
        }
        
        let diff = frame_seq_i32 - self.last_input_sequence;
        if diff > 1 {
            for index in (self.last_input_sequence + 1)..frame_seq_i32 {
                if !self.received_frame_sequences.contains(&(index as u32)) {
                    if self.debug {
                        println!("Detected missing sequence: {}", index);
                    }
                    self.lost_frame_sequences.insert(index as u32);
                }
            }
        }
        
        if frame_seq_i32 > self.last_input_sequence {
            self.last_input_sequence = frame_seq_i32;
        }
        
        for frame in &frameset.frames {
            if self.debug {
                println!("Processing frame {} from sequence {}", 
                        frame.sequence_frame_index.unwrap_or(0), frameset.sequence);
            }
            self.on_frame(frame)?;
        }
        Ok(())
    }

    fn on_frame(&mut self, frame: &Frame) -> Result<(), FramerError> {
        if self.debug {
            println!("Processing frame: reliability={:?}, ordered={}, split={}, payload_len={}", 
                frame.reliability,
                frame.reliability.is_ordered(),
                frame.is_split(),
                frame.payload.len());
            
            if frame.payload[0] == 0xfe {
                println!("Encrypted packet - First bytes: {:02x?}", &frame.payload[..4]);
            }
        }
        
        if frame.is_split() {
            self.on_split_frame(frame)?; 
        } else if frame.reliability.is_sequenced() {
            self.on_sequenced_frame(frame)?;
        } else if frame.reliability.is_ordered() {
            self.on_ordered_frame(frame)?;
        } else {
            self.process_packet(frame)?;
        }
        Ok(())
    }

    pub fn tick(&mut self) {
        let sequences: Vec<u32> = self.received_frame_sequences.clone()
            .into_iter()
            .collect();
        
        if !sequences.is_empty() {
            if self.debug {
                println!("Sending ACK for sequences: {:?}", sequences);
            }
            self.received_frame_sequences.clear();
            
            let ack = Ack {
                sequences,
            };
            let packet = ack.serialize().unwrap();
            let _ = self.socket.send(&packet, &format!("{}:{}", self.address, self.port));
        }

        let nack_sequences: Vec<u32> = self.lost_frame_sequences.clone()
            .into_iter()
            .collect();
        
        if !nack_sequences.is_empty() {
            self.lost_frame_sequences.clear();
            
            let nack = Nack {
                sequences: nack_sequences,
            };
            let packet = nack.serialize().unwrap();
            let _ = self.socket.send(&packet, &format!("{}:{}", self.address, self.port));
        }

        let _ = self.send_queue(self.output_frames.len() as u32);
    }

    pub fn process_packet(&mut self, frame: &Frame) -> Result<(), FramerError> {
        let id = frame.payload[0];
        
        if self.debug {
            println!("Processing packet with ID: 0x{:02x}", id);
            println!("Payload length: {}", frame.payload.len());
            
            if id == 0xfe {
                println!("Encrypted packet details:");
                println!("  First 8 bytes: {:02x?}", &frame.payload[..8.min(frame.payload.len())]);
                println!("  Last 8 bytes: {:02x?}", &frame.payload[frame.payload.len().saturating_sub(8)..]);
            }
        }
        
        if !frame.is_split() && self.received_packets.iter().any(|p| p == &frame.payload) {
            if self.debug {
                println!("Skipping duplicate packet with ID: {:?}", id);
            }
            return Ok(());
        }
        
        if !frame.is_split() {
            self.received_packets.push(frame.payload.clone());
            if self.debug {
                // println!("Processed new packet with ID: {:?}", id);
            }
        }
        
        match id {
            0x00 => {
                let ping = ConnectedPing::deserialize(&frame.payload).unwrap();
                let opacket = ConnectedPong {
                    ping_time: ping.timestamp,
                    pong_time: Utc::now().timestamp(),
                };
                let packet = opacket.serialize().unwrap();
                self.frame_and_send(packet, Priority::Immediate);
            }
            0x10 => {
                let conn = ConnectionRequestAccepted::deserialize(&frame.payload).unwrap();
                let timestamp = Utc::now().timestamp();
                
                let opacket = NewIncomingConnection {
                    server_address: Address::new(self.address.clone(), self.port, 4),
                    client_addresses: Address::new(
                        "127.0.0.1".to_string(), 
                        self.socket.socket.local_addr().unwrap().port(), 
                        4
                    ),
                    client_send_timestamp: timestamp,
                    server_send_timestamp: conn.server_send_time,
                };
                let packet = opacket.serialize().unwrap();
                self.frame_and_send(packet, Priority::Immediate);
            }
            0xc0 => {
                if self.debug {
                    println!("Received ACK");
                }
                let ack = Ack::deserialize(&frame.payload).unwrap();
                for sequence in ack.sequences {
                    if self.debug {
                        println!("Removing sequence: {}", sequence);
                    }
                    self.output_backup.remove(&sequence);
                }
            }
            0xa0 => {
                let nack = Nack::deserialize(&frame.payload).unwrap();
                for seq in nack.sequences {
                    if self.output_backup.contains_key(&seq) {
                        let frames = self.output_backup.remove(&seq).unwrap();
                        for frame in frames {
                            let mut mframe = frame.clone();
                            self.send_frame(&mut mframe, Priority::Immediate)?;
                        }
                        self.output_backup.remove(&seq);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn frame_and_send(&mut self, packet: Vec<u8>, priority: Priority) {
        let mut frame = Frame {
            reliability: Reliability::ReliableOrdered,
            reliable_frame_index: None,
            sequence_frame_index: None,
            ordered_frame_index: None,
            order_channel: Some(0),
            split_frame_index: None,
            split_size: None,
            split_id: None,
            payload: packet,
        };
        self.send_frame(&mut frame, priority).unwrap();
    }

    fn on_ordered_frame(&mut self, frame: &Frame) -> Result<(), FramerError> {
        let channel = frame.order_channel.unwrap_or(0) as usize;
        let expected_order_index = self.input_order_index[channel];
        let order_channel = frame.order_channel.unwrap_or(0) as u32;
        
        if self.debug {
            println!("Ordered frame: channel={}, index={}, expected={}", 
                    channel, frame.ordered_frame_index.unwrap_or(0), expected_order_index);
        }
        
        if frame.ordered_frame_index.unwrap_or(0) == expected_order_index {
            self.process_packet(frame)?;
            self.input_order_index[channel] += 1;
            
            let mut processed_count = 0;
            let mut next_index = self.input_order_index[channel];
            
            loop {
                let queue = self.input_ordering_queue.get_mut(&order_channel);
                let next_frame = match queue {
                    Some(q) => q.remove(&next_index),
                    None => None,
                };
                
                match next_frame {
                    Some(frame) => {
                        if self.debug {
                            println!("Processing queued ordered frame: index={}", next_index);
                        }
                        self.process_packet(&frame)?;
                        self.input_order_index[channel] += 1;
                        next_index += 1;
                        processed_count += 1;
                    },
                    None => break,
                }
                
                if processed_count > 1000 {
                    if self.debug {
                        println!("Warning: Processed too many queued frames, breaking loop");
                    }
                    break;
                }
            }
        } else if frame.ordered_frame_index.unwrap_or(0) > expected_order_index {
            if self.debug {
                println!("Queueing out-of-order frame: got={}, expected={}", 
                        frame.ordered_frame_index.unwrap_or(0), expected_order_index);
            }
            
            self.input_ordering_queue
                .entry(order_channel)
                .or_default()
                .insert(frame.ordered_frame_index.unwrap_or(0), frame.clone());
        } else {
            if self.debug {
                println!("Discarding old ordered frame: got={}, expected={}", 
                        frame.ordered_frame_index.unwrap_or(0), expected_order_index);
            }
        }
        Ok(())
    }

    fn on_split_frame(&mut self, frame: &Frame) -> Result<(), FramerError> {
        let split_id = frame.split_id.unwrap_or(0) as u32;
        
        if self.debug {
            println!("Processing split frame: id={}, index={}/{}, size={}, reliability={:?}", 
                split_id,
                frame.split_frame_index.unwrap_or(0),
                frame.split_size.unwrap_or(0),
                frame.payload.len(),
                frame.reliability);
        }
        
        let fragment = self.fragments_queue
            .entry(split_id)
            .or_default();
        
        let split_index = frame.split_frame_index.unwrap_or(0);
        if fragment.contains_key(&split_index) {
            if self.debug {
                println!("Skipping duplicate split frame fragment: id={}, index={}", split_id, split_index);
            }
            return Ok(());
        }
        
        fragment.insert(split_index, frame.clone());
        
        let split_size = frame.split_size.unwrap_or(0);
        if fragment.len() as u32 == split_size {
            if self.debug {
                println!("Reassembling split packet: id={}, total_parts={}, ordered={}", 
                    split_id, split_size, frame.reliability.is_ordered());
            }

            let mut stream = BinaryStream::new(None, None);
            let mut ordered_frame_index = None;
            let mut order_channel = None;
            
            for index in 0..split_size {
                if let Some(fragment_frame) = fragment.get(&index) {
                    stream.write_bytes(fragment_frame.payload.clone());
                    
                    if index == 0 {
                        ordered_frame_index = fragment_frame.ordered_frame_index;
                        order_channel = fragment_frame.order_channel;
                    }
                } else {
                    if self.debug {
                        println!("Missing fragment {} for split packet {}", index, split_id);
                    }
                    return Ok(());
                }
            }

            self.fragments_queue.remove(&split_id);

            let reassembled_frame = Frame {
                reliability: frame.reliability.clone(),
                reliable_frame_index: frame.reliable_frame_index,
                sequence_frame_index: frame.sequence_frame_index,
                ordered_frame_index,
                order_channel,
                split_frame_index: None,
                split_size: None,
                split_id: None,
                payload: stream.binary,
            };
            
            if self.debug {
                println!("Reassembled split packet: size={}, ordered_index={:?}, channel={:?}", 
                    reassembled_frame.payload.len(),
                    ordered_frame_index,
                    order_channel);
            }
            
            if frame.reliability.is_ordered() {
                self.on_ordered_frame(&reassembled_frame)?;
            } else {
                self.process_packet(&reassembled_frame)?;
            }
        }
        Ok(())
    }

    fn on_sequenced_frame(&mut self, frame: &Frame) -> Result<(), FramerError> {
        let channel = frame.order_channel.unwrap_or(0) as usize;
        let current_highest_sequence = self.input_highest_sequence_index[channel];
        if frame.sequence_frame_index.unwrap_or(0) > current_highest_sequence {
            self.input_highest_sequence_index[channel] = frame.sequence_frame_index.unwrap_or(0);
            self.process_packet(frame)?;
        }
        Ok(())
    }

    pub fn send_frame(&mut self, frame: &mut Frame, priority: Priority) -> Result<(), FramerError> {
        let channel = frame.order_channel.unwrap_or(0) as usize;
        
        if frame.reliability.is_sequenced() {
            frame.ordered_frame_index = Some(self.output_order_index[channel]);
            frame.sequence_frame_index = Some(self.output_sequence_index[channel]);
            self.output_sequence_index[channel] = self.output_sequence_index[channel] + 1;
        } else if frame.reliability.is_ordered() {
            let current_order_index = self.output_order_index[channel];
            frame.ordered_frame_index = Some(current_order_index);
            self.output_order_index[channel] = current_order_index + 1;
            self.output_sequence_index[channel] = 0;
        }

        let max_size = self.mtu_size - 36;
        
        if frame.payload.len() > max_size.into() {
            self.handle_large_payload(frame, max_size)?;
        } else {
            if frame.reliability.is_reliable() {
                frame.reliable_frame_index = Some(self.output_reliable_index);
                self.output_reliable_index += 1;
            }
            self.queue_frame(frame, priority)?;
        }

        // println!("Packet {:?}", frame.payload);
        Ok(())
    }

    fn handle_large_payload(&mut self, frame: &Frame, max_size: u16) -> Result<(), FramerError> {
        let payload_len = frame.payload.len();
        let max_chunk_size = max_size as usize;
        let split_size = (payload_len as f64 / max_chunk_size as f64).ceil() as usize;
        let split_id = self.output_split_index;
        self.output_split_index = (self.output_split_index + 1) % 65_536;
        
        if self.debug {
            println!("Splitting large packet: size={}, chunks={}, id={}", 
                payload_len, split_size, split_id);
        }
        
        for split_index in 0..split_size {
            let start = split_index * max_chunk_size;
            let end = (start + max_chunk_size).min(payload_len);
            
            let split_frame = Frame {
                reliability: frame.reliability.clone(),
                reliable_frame_index: Some(self.output_reliable_index),
                sequence_frame_index: frame.sequence_frame_index,
                ordered_frame_index: frame.ordered_frame_index,
                order_channel: frame.order_channel,
                split_frame_index: Some(split_index as u32),
                split_id: Some(split_id as u16),
                split_size: Some(split_size as u32),
                payload: frame.payload[start..end].to_vec(),
            };
            
            if split_frame.reliability.is_reliable() {
                self.output_reliable_index += 1;
            }

            if self.debug {
                println!("Sending split frame: id={}, index={}/{}, size={}", 
                    split_id, split_index, split_size, end-start);
            }

            self.queue_frame(&split_frame, Priority::Immediate)?;
        }
        Ok(())
    }

    pub fn queue_frame(&mut self, frame: &Frame, priority: Priority) -> Result<(), FramerError> {
        let mut total_length = 4;
        for qframe in self.output_frames.iter() {
            total_length += qframe.get_size();
        }
        
        if total_length + frame.get_size() > self.mtu_size as usize {
            self.send_queue(self.output_frames.len() as u32)?;
            self.output_frames.clear();
        }
        
        self.output_frames.insert(frame.clone());
        
        if priority == Priority::Immediate {
            self.send_queue(1)?;
        }
        
        Ok(())
    }

    fn send_queue(&mut self, length: u32) -> Result<(), FramerError> {
        if self.output_frames.len() == 0 {
            return Ok(());
        }

        let frames: Vec<Frame> = self.output_frames.iter()
            .take(length as usize)
            .cloned()
            .collect();

        let frameset = FrameSet {
            sequence: self.output_sequence,
            frames: frames.clone(),
        };
        self.output_sequence += 1;

        self.output_backup.insert(frameset.sequence, frames.clone());
        for frame in &frames {
            self.output_frames.remove(frame);
        }

        let serialized = frameset.serialize().unwrap();
        let _ = self.socket.send(&serialized, &format!("{}:{}", self.address, self.port));
        Ok(())
    }

    pub fn get_received_packets(&mut self) -> Vec<Vec<u8>> {
        let packets = self.received_packets.clone();
        self.received_packets.clear();
        packets
    }

    pub fn add_received_packet(&mut self, packet: Vec<u8>) {
        self.received_packets.push(packet);
    }

    pub fn get_next_packet(&mut self) -> Option<Vec<u8>> {
        if !self.received_packets.is_empty() {
            Some(self.received_packets.remove(0))
        } else {
            None
        }
    }

}
