//! Driver for the [SparkFun Serial 7 Segment Display](https://github.com/sparkfun/Serial7SegmentDisplay/wiki/Serial-7-Segment-Display-Datasheet)
//!
//! This is compatible with `embedded-hal`.
//!
//! Right now, only the SPI interface is supported. In the future, support will be
//! added for I2C/TWI and UART interfaces

#![no_std]

use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::OutputPin;
use bitflags::bitflags;

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

    pub(crate) const CLEAR_DISPLAY: u8 = 0x76;

    pub(crate) const DECIMAL_CTL: u8 = 0x77;
    pub(crate) const CURSOR_CTL: u8 = 0x79;
    pub(crate) const BRIGHTNESS_CTL: u8 = 0x7A;

    pub(crate) const DIGIT_1_CTL: u8 = 0x7B;
    pub(crate) const DIGIT_2_CTL: u8 = 0x7C;
    pub(crate) const DIGIT_3_CTL: u8 = 0x7D;
    pub(crate) const DIGIT_4_CTL: u8 = 0x7E;

    pub(crate) const BAUD_RATE_CFG: u8 = 0x7F;
    pub(crate) const I2C_ADDR_CFG: u8 = 0x80;

    pub(crate) const FACTORY_RESET: u8 = 0x81;
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error<T> {
    SpimError(T),
    CursorOutOfRange,
    DigitOutOfRange,
}

pub struct SevSegSpim<SPIM, CS> {
    spim: SPIM,
    csn: CS,
}

impl<SPIM, CS> SevSegSpim<SPIM, CS>
    where
        SPIM: Write<u8>,
        CS:  OutputPin,
{
    /// Create a new SparkFun Serial Seven Segment display using a SPI (Master)
    /// port. The SPI port has a maximum frequency of 250kHz, and must be in Mode 0.
    pub fn new(spim: SPIM, csn: CS) -> Self {
        Self {
            spim,
            csn,
        }
    }

    /// Set the digit cursor to a particular location
    /// `col` may be 0..=3, from left to right.
    pub fn set_cursor(&mut self, col: u8) -> Result<(), Error<SPIM::Error>> {
        if col >= 4 {
            return Err(Error::CursorOutOfRange);
        }

        self.send(&[
            command::CURSOR_CTL,
            col,
        ])
    }

    /// Completely clear the display
    pub fn clear(&mut self) -> Result<(), Error<SPIM::Error>> {
        self.send(&[command::CLEAR_DISPLAY])
    }

    /// Write a digit to the curent cursor position. This also
    /// increments the cursor position
    pub fn write_digit(&mut self, digit: u8) -> Result<(), Error<SPIM::Error>> {
        if digit > 0x0F {
            return Err(Error::DigitOutOfRange);
        }

        self.send(&[digit])
    }

    /// Write the requested punctuation to the display. This does not take
    /// the current state into account, so any unset flags in `punct_flags`
    /// will turn the corresponding LEDs off.
    pub fn write_punctuation(&mut self, punct_flags: PunctuationFlags) -> Result<(), Error<SPIM::Error>> {
        self.send(&[
            command::DECIMAL_CTL,
            punct_flags.bits()
        ])
    }

    /// Write the requested digits to the display, starting at the current
    /// cursor position. Each digit must be in the range 0x0..=0xF, and up
    /// to 4 digits may be updated at once. The cursor is incremented after
    /// each digit
    pub fn write_digits(&mut self, digits: &[u8]) -> Result<(), Error<SPIM::Error>> {
        // Too many digits?
        if digits.len() > 4 {
            return Err(Error::CursorOutOfRange);
        }

        // Any digit too big?
        for d in digits {
            if *d > 0x0F {
                return Err(Error::DigitOutOfRange);
            }
        }

        self.send(digits)
    }

    /// Write the number to the display. The number will be left-filled
    /// with zeroes if necessary. After this function, the cursor
    /// will be at position 0.
    pub fn set_num(&mut self, num: u16) -> Result<(), Error<SPIM::Error>> {
        if num > 9999 {
            return Err(Error::DigitOutOfRange);
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

    fn send(&mut self, data: &[u8]) -> Result<(), Error<SPIM::Error>> {
        self.csn.set_low();

        let ret = self.spim
            .write(&data)
            .map_err(|e| Error::SpimError(e))
            .map(|_| ());

        self.csn.set_high();

        ret
    }
}
