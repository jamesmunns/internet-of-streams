#![no_main]
#![no_std]
#![allow(unused_imports)]

// Built in dependencies
use core::fmt::Write;

// Crates.io dependencies

use dwm1001::{
    self,
    nrf52832_hal::{
        delay::Delay,
        prelude::*,
        timer::Timer,
        gpio::{Pin, Output, PushPull, Level, p0::P0_17},
        rng::Rng,
        spim::{
            Spim,
            Pins,
            MODE_0,
            Frequency as SpimFreq,
        },
        nrf52832_pac::{
            TIMER0,
            SPIM2,
        },
    },
    dw1000::{
        mac::Address,
        mac::frame::{AddressMode, PanId, ShortAddress},
        DW1000 as DW,
        Ready,
    },
    new_dw1000,
    new_usb_uarte,
    UsbUarteConfig,
    DW_RST,
    block_timeout,
    embedded_hal::timer::CountDown,
};
use heapless::{String, Vec, consts::*};
use nb::{
    block,
    Error as NbError,
};
use rtfm::app;
use postcard::{from_bytes, to_vec};

// NOTE: Panic Provider
use panic_ramdump as _;

// Workspace dependencies
use protocol::DemoMessage;
use uarte_logger::Logger;
use utils::delay;
use embedded_timeout_macros::TimeoutError;

mod spark_ser7seg;
use crate::spark_ser7seg::{SevSegSpim, PunctuationFlags as Punc};


#[app(device = dwm1001::nrf52832_hal::nrf52832_pac)]
const APP: () = {
    static mut LED_RED_1: Pin<Output<PushPull>>     = ();
    static mut TIMER:     Timer<TIMER0>             = ();
    static mut LOGGER:    Logger                    = ();
    static mut RANDOM:    Rng                      = ();
    static mut DISPLAY:   SevSegSpim<Spim<SPIM2>, Pin<Output<PushPull>>> = ();

    #[init]
    fn init() {
        let timer = device.TIMER0.constrain();
        let pins = device.P0.split();
        let uarte0 = new_usb_uarte(
            device.UARTE0,
            pins.p0_05,
            pins.p0_11,
            UsbUarteConfig::default(),
        );

        // CS      J1.29   P0.03
        // CLK     J1.28   P0.04
        // MOSI    J1.27   P0.06
        // MISO    J1.26   P0.07

        let cs = pins.p0_03.into_push_pull_output(Level::High).degrade();

        let spim = Spim::new(
            device.SPIM2,
            Pins {
                sck: pins.p0_04.into_push_pull_output(Level::Low).degrade(),
                mosi: Some(pins.p0_06.into_push_pull_output(Level::Low).degrade()),
                miso: Some(pins.p0_07.into_floating_input().degrade()),
            },
            SpimFreq::K125,
            MODE_0,
            0,
        );

        let mut rng = device.RNG.constrain();
        let mut delay = Delay::new(core.SYST);

        DISPLAY = SevSegSpim::new(
            spim,
            cs
        );

        RANDOM = rng;
        LOGGER = Logger::new(uarte0);
        TIMER = timer;
        LED_RED_1 = pins.p0_14.degrade().into_push_pull_output(Level::High);
    }

    #[idle(resources = [TIMER, LED_RED_1, LOGGER, RANDOM, DISPLAY])]
    fn idle() -> ! {
        let mut ctr = 0;

        resources.DISPLAY.clear();

        // pub struct PunctuationFlags: u8 {
        //     const DOT_BETWEEN_1_AND_2        = 0b0000_0001;
        //     const DOT_BETWEEN_2_AND_3        = 0b0000_0010;
        //     const DOT_BETWEEN_3_AND_4        = 0b0000_0100;
        //     const DOT_RIGHT_OF_4             = 0b0000_1000;
        //     const DOTS_COLON                 = 0b0001_0000;
        //     const APOSTROPHE_BETWEEN_3_AND_4 = 0b0010_0000;
        // }

        resources.DISPLAY.write_punctuation(
            Punc::DOT_BETWEEN_1_AND_2
        ).unwrap();

        resources.TIMER.start(1_000_000u32);
        while resources.TIMER.wait().is_err() {}

        resources.DISPLAY.write_punctuation(
            Punc::DOT_BETWEEN_2_AND_3
        ).unwrap();

        resources.TIMER.start(1_000_000u32);
        while resources.TIMER.wait().is_err() {}

        resources.DISPLAY.write_punctuation(
            Punc::DOT_BETWEEN_3_AND_4
        ).unwrap();

        resources.TIMER.start(1_000_000u32);
        while resources.TIMER.wait().is_err() {}

        resources.DISPLAY.write_punctuation(
            Punc::DOT_RIGHT_OF_4
        ).unwrap();

        resources.TIMER.start(1_000_000u32);
        while resources.TIMER.wait().is_err() {}

        resources.DISPLAY.write_punctuation(
            Punc::DOTS_COLON
        ).unwrap();

        resources.TIMER.start(1_000_000u32);
        while resources.TIMER.wait().is_err() {}

        resources.DISPLAY.write_punctuation(
            Punc::APOSTROPHE_BETWEEN_3_AND_4
        ).unwrap();

        resources.TIMER.start(1_000_000u32);
        while resources.TIMER.wait().is_err() {}

        resources.DISPLAY.write_punctuation(
            Punc::empty()
        ).unwrap();

        loop {
            resources.TIMER.start(8_000u32);
            while resources.TIMER.wait().is_err() {}
            (*resources.LED_RED_1).set_low();

            resources.DISPLAY.set_num(
                ctr,
            ).unwrap();

            ctr += 1;

            if ctr >= 10000 {
                ctr = 0;
            }

            resources.TIMER.start(2_000u32);
            while resources.TIMER.wait().is_err() {}
            (*resources.LED_RED_1).set_high();
        }
    }
};

