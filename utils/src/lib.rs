#![no_std]

use nb::{
    block,
};

use nrf52832_hal::{
    prelude::*,
    timer::Timer,
};

pub fn delay<T>(timer: &mut Timer<T>, cycles: u32) where T: TimerExt {
    timer.start(cycles);
    block!(timer.wait()).expect("wait fail");
}
