#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_halt;

use cortex_m_rt::entry;
use dwm1001;
use nb::block;

use dwm1001::{
    nrf52832_hal::{
        prelude::*,
        timer::Timer,
    },
    DWM1001,
};


#[entry]
fn main() -> ! {
    let mut dwm1001 = DWM1001::take().unwrap();

    let mut timer = dwm1001.TIMER0.constrain();

    loop {
        dwm1001.leds.D12.enable();
        delay(&mut timer, 20_000); // 20ms
        dwm1001.leds.D12.disable();
        delay(&mut timer, 230_000); // 230ms
    }
}


fn delay<T>(timer: &mut Timer<T>, cycles: u32) where T: TimerExt {
    timer.start(cycles);
    block!(timer.wait()).unwrap();
}
