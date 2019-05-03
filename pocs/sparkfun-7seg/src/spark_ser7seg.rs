use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::OutputPin;
use bitflags::bitflags;

pub struct SevSegSpim<SPIM, CS> {
    spim: SPIM,
    csn: CS,
}

impl<SPIM, CS> SevSegSpim<SPIM, CS> {
    pub fn new(spim: SPIM, csn: CS) -> Self {
        Self {
            spim,
            csn,
        }
    }
}

bitflags! {
    /// A bit packed structure representing days of the week
    pub struct PunctuationFlags: u8 {
        const DOT_BETWEEN_1_AND_2        = 0b0000_0001;
        const DOT_BETWEEN_2_AND_3        = 0b0000_0010;
        const DOT_BETWEEN_3_AND_4        = 0b0000_0100;
        const DOT_RIGHT_OF_4             = 0b0000_1000;
        const DOTS_COLON                 = 0b0001_0000;
        const APOSTROPHE_BETWEEN_3_AND_4 = 0b0010_0000;
    }
}

mod command {
    #![allow(dead_code)]

    pub const CLEAR_DISPLAY: u8 = 0x76;

    /// This is punctuation
    pub const DECIMAL_CTL: u8 = 0x77;
    pub const CURSOR_CTL: u8 = 0x79;
    pub const BRIGHTNESS_CTL: u8 = 0x7A;

    pub const DIGIT_1_CTL: u8 = 0x7B;
    pub const DIGIT_2_CTL: u8 = 0x7C;
    pub const DIGIT_3_CTL: u8 = 0x7D;
    pub const DIGIT_4_CTL: u8 = 0x7E;

    pub const BAUD_RATE_CFG: u8 = 0x7F;
    pub const I2C_ADDR_CFG: u8 = 0x80;

    pub const FACTORY_RESET: u8 = 0x81;
}

impl<SPIM, CS> SevSegSpim<SPIM, CS>
    where
        SPIM: Transfer<u8> + Write<u8>,
        CS:  OutputPin,
{
    pub fn set_cursor(&mut self, col: u8) -> Result<(), ()> {
        if col >= 4 {
            return Err(());
        }

        self.send(&[
            command::CURSOR_CTL,
            col,
        ])
    }

    pub fn clear(&mut self) -> Result<(), ()> {
        self.send(&[command::CLEAR_DISPLAY])
    }

    pub fn write_digit(&mut self, digit: u8) -> Result<(), ()> {
        if digit > 0x0F {
            return Err(());
        }

        self.send(&[digit])
    }

    pub fn write_punctuation(&mut self, punct_flags: PunctuationFlags) -> Result<(), ()> {
        self.send(&[
            command::DECIMAL_CTL,
            punct_flags.bits()
        ])
    }

    pub fn write_digits(&mut self, digits: &[u8]) -> Result<(), ()> {
        // Too many digits?
        if digits.len() > 4 {
            return Err(());
        }

        // Any digit too big?
        for d in digits {
            if *d > 0x0F {
                return Err(());
            }
        }

        self.send(digits)
    }

    pub fn set_num(&mut self, num: u16) -> Result<(), ()> {
        if num > 9999 {
            return Err(());
        }

        self.set_cursor(0)?;

        let data: [u8; 4] = [
            (num / 1000) as u8,
            ((num % 1000) / 100) as u8,
            ((num % 100) / 10) as u8,
            (num % 10) as u8,
        ];

        self.send(&data)
    }

    fn send(&mut self, data: &[u8]) -> Result<(), ()> {
        self.csn.set_low();

        let ret = self.spim
            .write(&data)
            .map_err(|_| ())
            .map(|_| ());

        self.csn.set_high();

        ret
    }
}
