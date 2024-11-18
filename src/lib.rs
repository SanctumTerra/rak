#![allow(dead_code)]
#![allow(unused_imports)]

pub mod client;
pub mod binarystream;
pub mod socket;
pub mod packets;

use crate::client::Client;
use napi_derive::*;
use napi::bindgen_prelude::*;

#[napi]
pub struct RaknetClient {
    client: Client,
}

#[napi]
impl RaknetClient {
    #[napi(constructor)]
    pub fn constructor(ip: Option<String>, port: Option<u16>, mtu_size: Option<u16>) -> Self {
        Self { client: Client::new(ip, port, mtu_size).unwrap() }
    }

    #[napi]
    pub fn connect(&mut self) -> Result<()> {
        self.client.connect()
            .map_err(|e| Error::from_reason(format!("Connection error: {:?}", e)))
    }

    #[napi]
    pub fn tick(&mut self) -> Result<()> {
        self.client.tick()
            .map_err(|e| Error::from_reason(format!("Tick error: {:?}", e)))
    }

    #[napi]
    pub fn ping(&mut self) -> Result<()> {
        self.client.ping()
            .map_err(|e| Error::from_reason(format!("Ping error: {:?}", e)))
    }

    #[napi]
    pub fn receive(&mut self) -> Result<Buffer> {
        self.client.get_received_packets()
            .map(|packets| {
                packets.first()
                    .cloned()
                    .unwrap_or_default()
            })
            .map(Buffer::from)
            .map_err(|e| Error::from_reason(format!("Receive error: {:?}", e)))
    }
    
    #[napi] 
    pub fn frame_and_send(&mut self, buffer: Buffer) -> Result<()> {
        self.client.frame_and_send(&buffer)
            .map(|_| ())
            .map_err(|e| Error::from_reason(format!("Frame and send error: {:?}", e)))
    }

    #[napi]
    pub fn send(&mut self, buffer: Buffer) -> Result<()> {
        self.client.send(&buffer)
            .map(|_| ())
            .map_err(|e| Error::from_reason(format!("Send error: {:?}", e)))
    }
}
