#![no_std]

#[allow(unused_imports)]
use serde;

use serde_derive::{Deserialize, Serialize};
// use nrf52832_hal::rng::Rng;

#[derive(Serialize, Deserialize, Debug)]
pub enum AllMessages {

}

#[derive(Serialize, Deserialize, Debug)]
pub enum LogOnLine<'a> {
    Log(&'a str),
    Warn(&'a str),
    Error(&'a str),
    BinaryRaw(&'a [u8]),
    ProtocolMessage(AllMessages),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DemoMessage {
    pub small:  u8,
    pub medium: u32,
    pub large: u64,
    pub text_bytes: [u8; 32],
}
