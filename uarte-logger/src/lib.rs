#![no_std]

use core::marker::PhantomData;
use core::ops::Deref;

use nrf52832_hal::{
    uarte::Uarte,
    target_constants::EASY_DMA_SIZE,
    nrf52832_pac::{
        UARTE0,
    },
};

use postcard::to_vec;
use heapless::{
    ArrayLength,
    Vec,
};

use protocol::AllMessages;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum LogOnLine<'a> {
    Log(&'a str),
    Warn(&'a str),
    Error(&'a str),
    BinaryRaw(&'a [u8]),
    ProtocolMessage(AllMessages),
}

pub struct Logger<BUFSZ>
where
    BUFSZ: ArrayLength<u8>,
{
    uart: Uarte<UARTE0>,
    _scratch_sz: PhantomData<BUFSZ>
}

impl<BUFSZ> Logger<BUFSZ>
where
    BUFSZ: ArrayLength<u8>,
 {
    pub fn new(uart: Uarte<UARTE0>) -> Self {
        Self {
            uart,
            _scratch_sz: PhantomData,
        }
    }

    pub fn log(&mut self, data: &str) -> Result<(), ()> {
        self.send(&LogOnLine::Log(data))
    }

    pub fn warn(&mut self, data: &str) -> Result<(), ()> {
        self.send(&LogOnLine::Warn(data))
    }

    pub fn error(&mut self, data: &str) -> Result<(), ()> {
        self.send(&LogOnLine::Error(data))
    }

    fn send(&mut self, msg: &LogOnLine) -> Result<(), ()> {
        let out: Vec<u8, BUFSZ> = to_vec(msg).map_err(|_| ())?;

        // Remove once nrf52832_hal reaches 0.8.0
        for c in out.deref().chunks(EASY_DMA_SIZE) {
            self.uart.write(c).map_err(|_| ())?;
        }
        Ok(())
    }
}
