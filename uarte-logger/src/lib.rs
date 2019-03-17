#![no_std]

use nrf52832_hal::{
    uarte::Uarte,
    target_constants::EASY_DMA_SIZE,
    nrf52832_pac::{
        UARTE0,
    },
};

pub struct Logger {
    uart: Uarte<UARTE0>,
    scratch: [u8; EASY_DMA_SIZE],
}

impl Logger {
    pub fn new(uart: Uarte<UARTE0>) -> Self {
        Self {
            uart,
            scratch: [0u8; EASY_DMA_SIZE],
        }
    }

    pub fn log(&mut self, data: &str) -> Result<(), ()> {
        self.send("LOG: ".as_bytes())?;
        self.send(data.as_bytes())?;
        self.send("\r\n".as_bytes())
    }

    pub fn warn(&mut self, data: &str) -> Result<(), ()> {
        self.send("WRN: ".as_bytes())?;
        self.send(data.as_bytes())?;
        self.send("\r\n".as_bytes())
    }

    pub fn error(&mut self, data: &str) -> Result<(), ()> {
        self.send("ERR: ".as_bytes())?;
        self.send(data.as_bytes())?;
        self.send("\r\n".as_bytes())
    }

    fn send(&mut self, buf: &[u8]) -> Result<(), ()> {
        for c in buf.chunks(EASY_DMA_SIZE) {
            self.scratch[..c.len()]
                .copy_from_slice(c);
            self.uart.write(
                &self.scratch[..c.len()]
            ).map_err(|_| ())?;
        }
        Ok(())
    }
}
