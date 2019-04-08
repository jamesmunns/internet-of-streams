#![no_std]

use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

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

use protocol::{AllMessages, LogOnLine};
use serde::{Serialize, Deserialize};

use cobs::{
    max_encoding_length,
    encode,
};

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
    pub fn new(mut uart: Uarte<UARTE0>) -> Self {
        // Send termination character
        uart.write(&[0x00]);

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
        let mut encoded: Vec<u8, BUFSZ> = Vec::new();
        {
            let out: Vec<u8, BUFSZ> = to_vec(msg).map_err(|_| ())?;
            encoded.resize(max_encoding_length(out.len()), 0x00)
                .map_err(|_| ())?;
            let sz = encode(out.deref(), encoded.deref_mut());
            encoded.truncate(sz);

            // Add message termination character
            encoded.push(0);

            // Okay, we can drop `out` now
        }

        // Remove once nrf52832_hal reaches 0.8.0
        for c in encoded.deref().chunks(EASY_DMA_SIZE) {
            self.uart.write(c).map_err(|_| ())?;
        }
        Ok(())
    }
}
