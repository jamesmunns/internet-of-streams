#![allow(dead_code)]

use dwm1001::nrf52832_hal::nrf52832_pac::{
    rtc0,
    RTC0,
};
use core::ops::Deref;

pub struct Stopped;
pub struct Started;

pub struct Rtc<T, M> {
    periph: T,
    mode: M,
}

pub trait RtcExt : Deref<Target=rtc0::RegisterBlock> + Sized {
    fn constrain(self) -> Rtc<Self, Stopped>;
}

impl<T> RtcExt for T where T: RtcExt {
    fn constrain(self) -> Rtc<T, Stopped> {
        Rtc {
            periph: self,
            mode: Stopped,
        }
    }
}

pub enum RtcInterrupt {
    Tick,
    Overflow,
    Compare0,
    Compare1,
    Compare2,
    Compare3,
}

pub enum RtcCompareReg {
    Compare0,
    Compare1,
    Compare2,
    Compare3,
}

impl<T, M> Rtc<T, M> where T: RtcExt {
    pub fn enable_counter(self) -> Rtc<T, Started> {
        Rtc {
            periph: self.periph,
            mode: Started,
        }
    }

    pub fn disable_counter(self) -> Rtc<T, Stopped> {
        Rtc {
            periph: self.periph,
            mode: Stopped,
        }
    }

    pub fn enable_interrupt(&mut self, int: RtcInterrupt) {
        use RtcInterrupt::*;
        match int {
            Tick => self.periph.intenset.write(|w| w.tick().set()),
            Overflow => self.periph.intenset.write(|w| w.ovrflw().set()),
            Compare0 => self.periph.intenset.write(|w| w.compare0().set()),
            Compare1 => self.periph.intenset.write(|w| w.compare1().set()),
            Compare2 => self.periph.intenset.write(|w| w.compare2().set()),
            Compare3 => self.periph.intenset.write(|w| w.compare3().set()),
        }
    }

    pub fn disable_interrupt(&mut self, int: RtcInterrupt) {
        use RtcInterrupt::*;
        match int {
            Tick => self.periph.intenclr.write(|w| w.tick().clear()),
            Overflow => self.periph.intenclr.write(|w| w.ovrflw().clear()),
            Compare0 => self.periph.intenclr.write(|w| w.compare0().clear()),
            Compare1 => self.periph.intenclr.write(|w| w.compare1().clear()),
            Compare2 => self.periph.intenclr.write(|w| w.compare2().clear()),
            Compare3 => self.periph.intenclr.write(|w| w.compare3().clear()),
        }
    }

    pub fn enable_event(&mut self, evt: RtcInterrupt) {
        use RtcInterrupt::*;
        match evt {
            Tick => self.periph.evtenset.write(|w| w.tick().set()),
            Overflow => self.periph.evtenset.write(|w| w.ovrflw().set()),
            Compare0 => self.periph.evtenset.write(|w| w.compare0().set()),
            Compare1 => self.periph.evtenset.write(|w| w.compare1().set()),
            Compare2 => self.periph.evtenset.write(|w| w.compare2().set()),
            Compare3 => self.periph.evtenset.write(|w| w.compare3().set()),
        }
    }

    pub fn disable_event(&mut self, evt: RtcInterrupt) {
        use RtcInterrupt::*;
        match evt {
            Tick => self.periph.evtenclr.write(|w| w.tick().clear()),
            Overflow => self.periph.evtenclr.write(|w| w.ovrflw().clear()),
            Compare0 => self.periph.evtenclr.write(|w| w.compare0().clear()),
            Compare1 => self.periph.evtenclr.write(|w| w.compare1().clear()),
            Compare2 => self.periph.evtenclr.write(|w| w.compare2().clear()),
            Compare3 => self.periph.evtenclr.write(|w| w.compare3().clear()),
        }
    }

    pub fn get_event(&mut self, evt: RtcInterrupt, clear_on_read: bool) -> bool {
        use RtcInterrupt::*;
        let mut orig = 0;
        let set_val = if clear_on_read { 1 } else { 0 };
        match evt {
            Tick => {
                self.periph.events_tick.modify(|r, w| {
                    unsafe { w.bits(set_val); }
                    orig = r.bits();
                    w
                })
            }
            Overflow => {
                self.periph.events_ovrflw.modify(|r, w| {
                    unsafe { w.bits(set_val); }
                    orig = r.bits();
                    w
                })
            }
            Compare0 => {
                self.periph.events_compare[0].modify(|r, w| {
                    unsafe { w.bits(set_val); }
                    orig = r.bits();
                    w
                })
            }
            Compare1 => {
                self.periph.events_compare[1].modify(|r, w| {
                    unsafe { w.bits(set_val); }
                    orig = r.bits();
                    w
                })
            }
            Compare2 => {
                self.periph.events_compare[2].modify(|r, w| {
                    unsafe { w.bits(set_val); }
                    orig = r.bits();
                    w
                })
            }
            Compare3 => {
                self.periph.events_compare[3].modify(|r, w| {
                    unsafe { w.bits(set_val); }
                    orig = r.bits();
                    w
                })
            }
        };

        orig == 1
    }

    pub fn set_compare(&mut self, reg: RtcCompareReg, val: u32) -> Result<(), Error> {
        if val >= (1 << 24) {
            return Err(Error::CompareOutOfRange);
        }

        use RtcCompareReg::*;
        let reg = match reg {
            Compare0 => 0,
            Compare1 => 1,
            Compare2 => 2,
            Compare3 => 3,
        };

        unsafe { self.periph.cc[reg].write(|w| w.bits(val)); }

        Ok(())
    }

    pub fn get_counter(&self) -> u32 {
        self.periph.counter.read().bits()
    }

    pub fn release(self) -> T {
        self.periph
    }
}

pub enum Error {
    PrescalerOutOfRange,
    CompareOutOfRange,
}

impl<T> Rtc<T, Stopped> where T: RtcExt {
    pub fn set_prescaler(&mut self, prescaler: u32) -> Result<(), Error> {
        if prescaler >= (1 << 12) {
            return Err(Error::PrescalerOutOfRange);
        }

        unsafe { self.periph.prescaler.write(|w| w.bits(prescaler)) };

        Ok(())
    }
}
