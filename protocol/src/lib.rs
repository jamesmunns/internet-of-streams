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

impl DemoMessage {
    // pub fn rand(rng: &mut Rng) -> Self {
    //     let start = (rng.random_u32() % ((MEME.len() - 32) as u32)) as usize;
    //     let mut strbuf = [0u8; 32];
    //     strbuf.copy_from_slice(&MEME.as_bytes()[start..(start+32)]);
    //     Self {
    //         small: rng.random_u8(),
    //         medium: rng.random_u32(),
    //         large: rng.random_u64(),
    //         text_bytes: strbuf
    //     }
    // }
}

pub const MEME: &str = "Did you ever hear the tragedy of Darth Plagueis The Wise? I thought not. It's not a story the Jedi would tell you. It's a Sith legend. Darth Plagueis was a Dark Lord of the Sith, so powerful and so wise he could use the Force to influence the midichlorians to create life... He had such a knowledge of the dark side that he could even keep the ones he cared about from dying. The dark side of the Force is a pathway to many abilities some consider to be unnatural. He became so powerful... the only thing he was afraid of was losing his power, which eventually, of course, he did. Unfortunately, he taught his apprentice everything he knew, then his apprentice killed him in his sleep. Ironic. He could save others from death, but not himself.";
