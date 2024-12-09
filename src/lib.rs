pub mod client;
pub use client::*;
pub mod proto;
pub use proto::*;
pub mod socket;
pub use socket::*;
pub mod binary_stream;
pub use binary_stream::*;

use napi_derive::*;
use napi::bindgen_prelude::*;

#[napi(object)]
pub struct JsEvent {
    pub name: String,
    pub data: Vec<u8>
}

#[napi]
pub struct RaknetClient {
    client: Client
}

#[napi]
impl RaknetClient {
    #[napi(constructor)]
    pub fn new(host: String, port: u32) -> Self {
        Self {
            client: Client::new(host, port as u16)
        }
    }

    #[napi]
    pub fn connect(&mut self) -> Result<()> {
        self.client.connect().map_err(|e| Error::from_reason(e))
    }

    #[napi]
    pub fn receive(&mut self) -> Result<Vec<u8>> {
        self.client.receive()
            .map_err(|e| Error::from_reason(e))
    }

    #[napi]
    pub fn frame_and_send(&mut self, data: Buffer) -> Result<()> {
        self.client.frame_and_send(data.to_vec());
        Ok(())
    }

    #[napi]
    pub fn tick(&mut self) -> Result<()> {
        self.client.tick();
        Ok(())
    }

    #[napi]
    pub fn ping(&mut self) -> Result<()> {
        self.client.ping();
        Ok(())
    }
    
    #[napi]
    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }
    
    #[napi(js_name = "onEvent")]
    pub fn on_event(&mut self) -> Result<Option<JsEvent>> {
        match self.client.event_receiver.try_recv() {
            Ok(event) => {
                Ok(Some(JsEvent {
                    name: event.name,
                    data: event.data,
                }))
            },
            Err(_) => Ok(None)
        }
    }
}