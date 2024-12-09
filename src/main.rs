mod binary_stream;

pub use binary_stream::*;
mod socket;
use chrono::Utc;
pub use socket::*;
mod proto;
pub use proto::*;
mod client;
pub use client::*;

// pub const MAGIC: [u8; 16] = [
//     0x00, 0xFF, 0xFF, 0x00,
//     0xFE, 0xFE, 0xFE, 0xFE,
//     0xFD, 0xFD, 0xFD, 0xFD,
//     0x12, 0x34, 0x56, 0x78
// ];


fn main() {
    let mut client = Client::new("135.148.137.229".to_string(), 19132);
    println!("Connecting to server... {:?}", Utc::now().timestamp());
    client.connect().unwrap();

    loop {
        client.tick();
        let _data = client.receive().unwrap();
        // println!("{:?}", data);
    }
}

// use std::time::{SystemTime, UNIX_EPOCH};


// fn main() {
//     let guid = 123123123123123;

//     let host = "135.148.137.229".to_string();
//     let socket = Socket::new(host.clone(), 19132);
    
//     let timestamp: i64 = match SystemTime::now().duration_since(UNIX_EPOCH) {
//         Ok(duration) => duration.as_millis() as i64,
//         Err(_) => panic!("Time went backwards"),
//     };
//     let ping = UnconnectedPing::new(timestamp, guid);
//     socket.send(ping.serialize()).unwrap();

//     loop { 
//         let mut buffer = [0; 1500];
//         socket.receive(&mut buffer).unwrap();
//         if buffer[0] == UnconnectedPong::ID {
//             let pong = UnconnectedPong::deserialize(buffer.to_vec());
//             println!("{:?}", pong);
//             let request = ConnectionRequestOne::new(11, 1492);
//             socket.send(request.serialize()).unwrap();
//         } else if buffer[0] == ConnectionReplyOne::ID {
//             let reply = ConnectionReplyOne::deserialize(buffer.to_vec()).unwrap();
//             println!("{:?}", reply);
//             let request = ConnectionRequestTwo::new(
//                 Address::new(4, host.clone(), 19132), reply.mtu_size, guid);
//             socket.send(request.serialize()).unwrap();
//         } else if buffer[0] == ConnectionReplyTwo::ID {
//             let reply = ConnectionReplyTwo::deserialize(buffer.to_vec()).unwrap();
//             println!("{:?}", reply);
//         } else {
//             println!("{:?}", buffer[0]);
//         }

//         std::thread::sleep(std::time::Duration::from_millis(20));
//     }
// }
