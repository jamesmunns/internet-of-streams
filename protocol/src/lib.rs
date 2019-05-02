#![no_std]

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DemoMessage<'a> {
    pub small:  u8,
    pub medium: u32,
    pub large: u64,
    pub text_bytes: &'a str,
}
