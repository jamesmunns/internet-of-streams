#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_halt;

use dwm1001;
use nb::block;
use rtfm::app;

use dwm1001::{
    nrf52832_hal::{
        prelude::*,
        timer::Timer,
        gpio::{Pin, Output, PushPull, Level},
        uarte::{Pins, Parity as UartParity, Baudrate as UartBaudrate},
    },
    // DWM1001,
    // Led,
};

use nrf52832_pac::{
    TIMER0,
};

mod logger;
use logger::Logger;

const MEME: &str = "Did you ever hear the tragedy of Darth Plagueis The Wise? I thought not. It's not a story the Jedi would tell you. It's a Sith legend. Darth Plagueis was a Dark Lord of the Sith, so powerful and so wise he could use the Force to influence the midichlorians to create life… He had such a knowledge of the dark side that he could even keep the ones he cared about from dying. The dark side of the Force is a pathway to many abilities some consider to be unnatural. He became so powerful… the only thing he was afraid of was losing his power, which eventually, of course, he did. Unfortunately, he taught his apprentice everything he knew, then his apprentice killed him in his sleep. Ironic. He could save others from death, but not himself.";

#[app(device = nrf52832_pac)]
const APP: () = {
    static mut LED_RED_1: Pin<Output<PushPull>>     = ();
    static mut TIMER:     Timer<TIMER0>             = ();
    static mut LOGGER:    Logger                    = ();

    #[init]
    fn init() {
        let timer = device.TIMER0.constrain();
        let pins = device.P0.split();
        let uarte0 = device.UARTE0.constrain(Pins {
                txd: pins.p0_05.into_push_pull_output(Level::High).degrade(),
                rxd: pins.p0_11.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            UartParity::EXCLUDED,
            UartBaudrate::BAUD1M,
        );

        LOGGER = Logger::new(uarte0);
        TIMER = timer;
        LED_RED_1 = pins.p0_14.degrade().into_push_pull_output(Level::High);
    }

    #[idle(resources = [TIMER, LED_RED_1, LOGGER])]
    fn idle() -> ! {

        loop {
            (*resources.LED_RED_1).set_low();
            delay(resources.TIMER, 500_000);
            (*resources.LED_RED_1).set_high();
            delay(resources.TIMER, 500_000);

            resources.LOGGER.log(MEME).unwrap();
        }
    }
};

fn delay<T>(timer: &mut Timer<T>, cycles: u32) where T: TimerExt {
    timer.start(cycles);
    block!(timer.wait()).unwrap();
}
