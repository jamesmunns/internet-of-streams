#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_halt;

use dwm1001;
use nb::block;
use rtfm::app;
use cortex_m;
use bbqueue::{
    BBQueue,
    cortex_m_bbq,
};

use dwm1001::{
    nrf52832_hal::{
        prelude::*,
        timer::Timer,
        gpio::{Pin, Output, PushPull, Level},
        uarte::{
            Pins,
            Parity as UartParity,
            Baudrate as UartBaudrate,
            nonblocking::{
                UarteAsync,
                UarteAsyncExt,
            },
        },
    },
    // DWM1001,
    // Led,
};

use nrf52832_pac::{
    TIMER0,
    UARTE0,
    NVIC,
};


#[app(device = nrf52832_pac)]
const APP: () = {
    static mut LED_RED_1: Pin<Output<PushPull>>     = ();
    static mut TIMER:     Timer<TIMER0>             = ();
    static mut UARTE:     UarteAsync<UARTE0>             = ();

    #[init]
    fn init() {
        let timer = device.TIMER0.constrain();
        let pins = device.P0.split();

        let bbq = cortex_m_bbq!(2048).unwrap();
        let (p, c) = bbq.split();


        let uarte0: UarteAsync<UARTE0> = device.UARTE0.constrain_async(
            // TODO: NVIC goes here
            Pins {
                txd: pins.p0_05.into_push_pull_output(Level::High).degrade(),
                rxd: pins.p0_11.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            UartParity::EXCLUDED,
            UartBaudrate::BAUD460800,
            p,
            c,
        );

        UARTE = uarte0;
        TIMER = timer;
        LED_RED_1 = pins.p0_14.degrade().into_push_pull_output(Level::High);
    }

    #[idle(resources = [TIMER, LED_RED_1])]
    fn idle() -> ! {


        loop {
            (*resources.LED_RED_1).set_low();
            delay(resources.TIMER, 20_000); // 20ms
            (*resources.LED_RED_1).set_high();
            delay(resources.TIMER, 230_000); // 230ms
        }
    }
};

fn delay<T>(timer: &mut Timer<T>, cycles: u32) where T: TimerExt {
    timer.start(cycles);
    block!(timer.wait()).unwrap();
}
