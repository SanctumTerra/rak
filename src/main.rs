#![allow(dead_code)]
#![allow(unused_imports)]
mod packets;
mod binarystream;
mod socket;
mod client;

use crate::socket::SocketError;
use crate::client::Client;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), SocketError> {
    println!("Raknet Client main");
    let start = std::time::Instant::now();
    // let ip = "135.148.137.229";
    let ip = "127.0.0.1";
    let client = Arc::new(Mutex::new(Client::new(Some(ip.to_string()), Some(19132), Some(1492))?));
    
    let receive_client = Arc::clone(&client);
    {
        let mut locked_client = client.lock().unwrap();
        locked_client.connect()?;
        locked_client.ping()?;
    }

    std::thread::spawn(move || {
        loop {
            if let Ok(mut locked_client) = receive_client.lock() {
                let result = locked_client.receive();
                if let Ok(data) = result {
                    if data[7] == 16 {
                        println!("Received 16");
                        println!("Time: {:?}", start.elapsed());
                    }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }}
    });

    loop {
        if let Ok(mut locked_client) = client.lock() {
            let _ = locked_client.tick();
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
