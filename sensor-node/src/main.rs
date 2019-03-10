#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_halt;

use dwm1001;
use nb::block;
use rtfm::app;

use dwm1001::{
    nrf52832_hal::{
        delay::Delay,
        prelude::*,
        timer::Timer,
        gpio::{Pin, Output, PushPull, Level},
        spim::{Spim, Pins as SpimPins},
        uarte::{Pins as UartePins, Parity as UartParity, Baudrate as UartBaudrate},
    },
    // DWM1001,
    // Led,
};

use nrf52832_pac::{
    TIMER0,
    SPIM2,
};

use dw1000::{DW1000 as DW};

mod logger;
mod dwm1001_local;
use logger::Logger;
use dwm1001_local::DW_RST;

const MEME: &str = "Did you ever hear the tragedy of Darth Plagueis The Wise? I thought not. It's not a story the Jedi would tell you. It's a Sith legend. Darth Plagueis was a Dark Lord of the Sith, so powerful and so wise he could use the Force to influence the midichlorians to create life… He had such a knowledge of the dark side that he could even keep the ones he cared about from dying. The dark side of the Force is a pathway to many abilities some consider to be unnatural. He became so powerful… the only thing he was afraid of was losing his power, which eventually, of course, he did. Unfortunately, he taught his apprentice everything he knew, then his apprentice killed him in his sleep. Ironic. He could save others from death, but not himself.";

#[app(device = nrf52832_pac)]
const APP: () = {
    static mut LED_RED_1: Pin<Output<PushPull>>     = ();
    static mut TIMER:     Timer<TIMER0>             = ();
    static mut LOGGER:    Logger                    = ();
    static mut DW1000:    DW<
                            Spim<SPIM2>,
                            Pin<Output<PushPull>>,
                            dw1000::Ready,
                          > = ();
    static mut DW_RST_PIN: DW_RST                   = ();

    #[init]
    fn init() {
        let timer = device.TIMER0.constrain();
        let pins = device.P0.split();
        let uarte0 = device.UARTE0.constrain(UartePins {
                txd: pins.p0_05.into_push_pull_output(Level::High).degrade(),
                rxd: pins.p0_11.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            UartParity::EXCLUDED,
            UartBaudrate::BAUD115200,
        );

        let spim2 = device.SPIM2.constrain(SpimPins {
            sck : pins.p0_16.into_push_pull_output(Level::Low).degrade(),
            mosi: Some(pins.p0_20.into_push_pull_output(Level::Low).degrade()),
            miso: Some(pins.p0_18.into_floating_input().degrade()),
        });
        let dw_cs = pins.p0_17.degrade().into_push_pull_output(Level::High);
        let dw1000 = DW::new(spim2, dw_cs);
        let mut rst_pin = DW_RST::new(pins.p0_24.into_floating_input());

        let clocks = device.CLOCK.constrain().freeze();

        let mut delay = Delay::new(core.SYST, clocks);

        rst_pin.reset_dw1000(&mut delay);

        let dw1000 = dw1000.init().unwrap();

        DW_RST_PIN = rst_pin;
        DW1000 = dw1000;
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
