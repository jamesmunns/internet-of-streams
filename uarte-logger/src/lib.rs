#![no_std]

use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use serde::Serialize;

use nrf52832_hal::{
    uarte::Uarte,
    target_constants::EASY_DMA_SIZE,
    nrf52832_pac::{
        UARTE0,
    },
};

use postcard::to_vec_cobs_ref;
use heapless::{
    ArrayLength,
    Vec,
};

use protocol::{LogOnLine};

pub struct Logger<BUFSZ, T>
where
    BUFSZ: ArrayLength<u8>,
{
    uart: Uarte<UARTE0>,
    vec: Vec<u8, BUFSZ>,
    _scratch_sz: PhantomData<BUFSZ>,
    _bin_data: PhantomData<T>
}

impl<BUFSZ, T> Logger<BUFSZ, T>
where
    BUFSZ: ArrayLength<u8>,
    T: Serialize,
 {
    pub fn new(mut uart: Uarte<UARTE0>) -> Self {
        // Send termination character
        uart.write(&[0x00]).unwrap();

        Self {
            uart,
            vec: Vec::new(),
            _scratch_sz: PhantomData,
            _bin_data: PhantomData,
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

    pub fn raw_bin(&mut self, data: &[u8]) -> Result<(), ()> {
        self.send(&LogOnLine::BinaryRaw(data))
    }

    pub fn data(&mut self, data: T) -> Result<(), ()> {
        self.send(&LogOnLine::ProtocolMessage(data))
    }

    fn send(&mut self, msg: &LogOnLine<T>) -> Result<(), ()> {
        to_vec_cobs_ref(msg, &mut self.vec).map_err(|_| ())?;
        // let encoded: Vec<u8, BUFSZ> = to_vec_cobs_ref(msg, &mut self.vec).map_err(|_| ())?;

        // Remove once nrf52832_hal reaches 0.8.0
        for c in self.vec.deref().chunks(EASY_DMA_SIZE) {
            self.uart.write(c).map_err(|_| ())?;
        }

        self.vec.clear();
        Ok(())
    }
}
