#![no_main]
#![no_std]

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
        spim::Spim,
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


const NOMINAL_WAIT_US: u32 = 400_000;
const MAX_WAIT_JITTER_US: u32 = 200_000;


#[app(device = dwm1001::nrf52832_hal::nrf52832_pac)]
const APP: () = {
    static mut LED_RED_1: Pin<Output<PushPull>>     = ();
    static mut TIMER:     Timer<TIMER0>             = ();
    static mut LOGGER:    Logger                    = ();
    static mut DW1000:    DW<
                            Spim<SPIM2>,
                            P0_17<Output<PushPull>>,
                            Ready,
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

        let mut rng = device.RNG.constrain();

        let dw1000 = new_dw1000(
            device.SPIM2,
            pins.p0_16,
            pins.p0_20,
            pins.p0_18,
            pins.p0_17,
            None,
        );

        let mut rst_pin = DW_RST::new(pins.p0_24.into_floating_input());
        let mut delay = Delay::new(core.SYST);

        rst_pin.reset_dw1000(&mut delay);

        let mut dw1000 = dw1000.init().unwrap();

        let pan_id = PanId::decode(&(0x0386u16.to_le_bytes())).unwrap().0;
        let saddr  = ShortAddress::decode(&rng.random_u16().to_le_bytes()).unwrap().0;

        let addr = Address::Short(
            pan_id,
            saddr,
        );

        loop {
            if dw1000.set_address(
                pan_id,
                saddr,
            ).is_err() {
                continue;
            }

            if let Ok(raddr) = dw1000.get_address() {
                if addr == raddr {
                    break;
                }
            }
        }

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
            let jitter = resources.RANDOM.random_u32() % MAX_WAIT_JITTER_US;
            resources.TIMER.start(NOMINAL_WAIT_US + jitter);
            let message = rand_msg(&mut resources.RANDOM);
            let serd: Vec<u8, U1024> = to_vec(&message).expect("ser fail");

            let mut tx_fut = resources.DW1000.send(
                &serd,
                Address::broadcast(&AddressMode::Short),
                None
            ).expect("tx fail");

            match block_timeout!(&mut *resources.TIMER, tx_fut.wait()) {
                Ok(_) => {
                    resources.LOGGER.log("Sent hello").expect("hello fail");
                },
                _ => continue,
            };



            let mut rx_fut = resources.DW1000.receive().expect("rx fut fail");

            match block_timeout!(&mut *resources.TIMER, rx_fut.wait(&mut scratch)) {
                Ok(msg) => {
                    match from_bytes::<DemoMessage>(msg.frame.payload) {
                        Ok(val) => {
                            let mut out: String<U256> = String::new();
                            write!(&mut out, "got message! \r\n").unwrap();
                            write!(&mut out, "small: {:016X}\r\n", val.small).unwrap();
                            write!(&mut out, "med:   {:016X}\r\n", val.medium).unwrap();
                            write!(&mut out, "large  {:016X}\r\n", val.large).unwrap();
                            write!(&mut out, "text: {}\r\n", &val.text_bytes).unwrap();
                            resources.LOGGER.log(&out).unwrap();
                        }
                        _ => {
                            resources.LOGGER.error("failed to deser").unwrap();
                        }
                    }

                    // Drain out the rest of the time
                    while resources.TIMER.wait().is_err() {}
                }
                Err(TimeoutError::Timeout) => {
                    resources.LOGGER.log("No Packet!").expect("no log fail");
                }
                Err(TimeoutError::Other(error)) => {
                    let mut out: String<U256> = String::new();
                    write!(&mut out, "rx fail: {:?}", error).unwrap();
                    resources.LOGGER.error(out.as_str()).unwrap();
                }
            };

            resources.DW1000.force_idle().expect("idle fail");
        }
    }
};


pub fn rand_msg(rng: &mut Rng) -> DemoMessage<'static> {
    let start = (rng.random_u32() % ((MEME.len() - 64) as u32)) as usize;
    let len = ((rng.random_u32() % 63) + 1) as usize;

    DemoMessage {
        small: rng.random_u8(),
        medium: rng.random_u32(),
        large: rng.random_u64(),
        text_bytes: &MEME[start..(start+len)],
    }
}


pub const MEME: &str = "Did you ever hear the tragedy of Darth Plagueis The Wise? I thought not. It's not a story the Jedi would tell you. It's a Sith legend. Darth Plagueis was a Dark Lord of the Sith, so powerful and so wise he could use the Force to influence the midichlorians to create life... He had such a knowledge of the dark side that he could even keep the ones he cared about from dying. The dark side of the Force is a pathway to many abilities some consider to be unnatural. He became so powerful... the only thing he was afraid of was losing his power, which eventually, of course, he did. Unfortunately, he taught his apprentice everything he knew, then his apprentice killed him in his sleep. Ironic. He could save others from death, but not himself.";
