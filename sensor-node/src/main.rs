#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_ramdump;

use dwm1001;
use nb::{
    block,
    Error as NbError,
};
use rtfm::app;

use dwm1001::{
    nrf52832_hal::{
        delay::Delay,
        prelude::*,
        timer::Timer,
        gpio::{Pin, Output, PushPull, Level, p0::P0_17},
        rng::Rng,
        spim::{Spim},
    },
    dw1000::{
        mac::Address,
    },
    new_dw1000,
    new_usb_uarte,
    UsbUarteConfig,
    DW_RST,
};

use nrf52832_pac::{
    TIMER0,
    SPIM2,
};

use heapless::{String, consts::*};

use dw1000::{DW1000 as DW};
use core::fmt::Write;

mod logger;
use logger::Logger;

#[allow(unused_imports)]
use serde;

use serde_derive::{Deserialize, Serialize};
use ssmarshal::{serialize, deserialize};


#[derive(Debug, Deserialize, Serialize)]
struct DemoMessage {
    small:  u8,
    medium: u32,
    large: u64,
    text_bytes: [u8; 32],
}

impl DemoMessage {
    fn rand(rng: &mut Rng) -> Self {
        let start = (rng.random_u32() % ((MEME.len() - 32) as u32)) as usize;
        let mut strbuf = [0u8; 32];
        strbuf.copy_from_slice(&MEME.as_bytes()[start..(start+32)]);
        Self {
            small: rng.random_u8(),
            medium: rng.random_u32(),
            large: rng.random_u64(),
            text_bytes: strbuf
        }
    }
}

const MEME: &str = "Did you ever hear the tragedy of Darth Plagueis The Wise? I thought not. It's not a story the Jedi would tell you. It's a Sith legend. Darth Plagueis was a Dark Lord of the Sith, so powerful and so wise he could use the Force to influence the midichlorians to create life... He had such a knowledge of the dark side that he could even keep the ones he cared about from dying. The dark side of the Force is a pathway to many abilities some consider to be unnatural. He became so powerful... the only thing he was afraid of was losing his power, which eventually, of course, he did. Unfortunately, he taught his apprentice everything he knew, then his apprentice killed him in his sleep. Ironic. He could save others from death, but not himself.";

#[app(device = nrf52832_pac)]
const APP: () = {
    static mut LED_RED_1: Pin<Output<PushPull>>     = ();
    static mut TIMER:     Timer<TIMER0>             = ();
    static mut LOGGER:    Logger                    = ();
    static mut DW1000:    DW<
                            Spim<SPIM2>,
                            P0_17<Output<PushPull>>,
                            dw1000::Ready,
                          > = ();
    static mut DW_RST_PIN: DW_RST                   = ();
    static mut RANDOM:     Rng                      = ();

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

        let rng = device.RNG.constrain();

        let dw1000 = new_dw1000(
            device.SPIM2,
            pins.p0_16,
            pins.p0_20,
            pins.p0_18,
            pins.p0_17,
        );

        let mut rst_pin = DW_RST::new(pins.p0_24.into_floating_input());

        let clocks = device.CLOCK.constrain().freeze();

        let mut delay = Delay::new(core.SYST, clocks);

        rst_pin.reset_dw1000(&mut delay);

        let dw1000 = dw1000.init().unwrap();

        RANDOM = rng;
        DW_RST_PIN = rst_pin;
        DW1000 = dw1000;
        LOGGER = Logger::new(uarte0);
        TIMER = timer;
        LED_RED_1 = pins.p0_14.degrade().into_push_pull_output(Level::High);
    }

    #[idle(resources = [TIMER, LED_RED_1, LOGGER, RANDOM, DW1000])]
    fn idle() -> ! {
        let mut scratch = [0u8; 4096];
        loop {
            let message = DemoMessage::rand(&mut resources.RANDOM);

            let sz = serialize(&mut scratch, &message).expect("ser fail");

            block!(resources.DW1000.send(
                &scratch[..sz],
                Address::broadcast(),
                None
            ).expect("tx fail").wait()).expect("tx fail block");
            resources.LOGGER.log("Sent hello").expect("hello fail");

            let mut rx_fut = resources.DW1000.receive().expect("rx fut fail");

            let a_time = 250_000 + (resources.RANDOM.random_u32() & 0x7_FFFF);
            let b_time = 250_000 + (resources.RANDOM.random_u32() & 0x7_FFFF);

            (*resources.LED_RED_1).set_low();
            delay(resources.TIMER, a_time);
            (*resources.LED_RED_1).set_high();
            delay(resources.TIMER, b_time);


            match rx_fut.wait(&mut scratch) {
                Ok(msg) => {
                    match deserialize::<DemoMessage>(msg.frame.payload) {
                        Ok((val, _)) => {
                            let mut out: String<U256> = String::new();
                            write!(&mut out, "got message! \r\n").unwrap();
                            write!(&mut out, "small: {:016X}\r\n", val.small).unwrap();
                            write!(&mut out, "med:   {:016X}\r\n", val.medium).unwrap();
                            write!(&mut out, "large  {:016X}\r\n", val.large).unwrap();
                            write!(&mut out, "text: {}\r\n", ::core::str::from_utf8(&val.text_bytes).unwrap()).unwrap();
                            resources.LOGGER.log(out.as_str()).unwrap();
                        }
                        _ => {
                            resources.LOGGER.error("failed to deser").unwrap();
                        }
                    }
                },
                Err(NbError::WouldBlock) => {
                    resources.LOGGER.log("No Packet!").expect("no log fail");
                },
                Err(e) => {
                    let mut out: String<U256> = String::new();
                    write!(&mut out, "rx fail: {:?}", e).unwrap();
                    resources.LOGGER.error(out.as_str()).unwrap();
                }
            }

            resources.DW1000.force_idle().expect("idle fail");
        }
    }
};

fn delay<T>(timer: &mut Timer<T>, cycles: u32) where T: TimerExt {
    timer.start(cycles);
    block!(timer.wait()).expect("wait fail");
}
