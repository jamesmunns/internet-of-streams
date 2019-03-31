#![no_main]
#![no_std]

// Built in dependencies
use core::fmt::Write;

// Crates.io dependencies
use dw1000::{DW1000 as DW};
use dwm1001::{
    self,
    nrf52832_hal::{
        // delay::Delay,
        prelude::*,
        timer::Timer,
        gpio::{Pin, Output, PushPull, Level, p0::P0_17},
        rng::Rng,
        spim::{Spim},
        nrf52832_pac::{
            TIMER0,
            SPIM2,
            RTC0 as RTC0_PERIPHERAL,
        },
    },
    new_dw1000,
    new_usb_uarte,
    UsbUarteConfig,
    DW_RST,
};
use heapless::{String, consts::*};
use rtfm::app;

// NOTE: Panic Provider
use panic_ramdump as _;

// NOTE: Must explicitly pull in for RTFM
use nrf52832_pac;

// Workspace dependencies
use uarte_logger::Logger;

use nrf52_hal_backports::{
    clocks::{
        ClocksExt,
        LfOscConfiguration
    },
    rtc::{
        Rtc,
        RtcExt,
        Started,
        RtcInterrupt
    },
    delay::Delay,
};

use uhr::{
    Uhr,
    Wecker,
    FixedOffsetFromUtc,
    UnixTimestamp,
    DayFlags,
};

use core::time::Duration;


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
    static mut RTCT:        Rtc<RTC0_PERIPHERAL, Started>       = ();
    static mut ALARM_CLOCK: Wecker<U8> = ();

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

        // Start the clocks
        let _clocks = device
            .CLOCK
            .constrain()
            .enable_ext_hfosc()
            .set_lfclk_src_external(LfOscConfiguration::NoExternalNoBypass)
            .start_lfclk();


        let mut delay = Delay::new(core.SYST);

        rst_pin.reset_dw1000(&mut delay);

        let dw1000 = dw1000.init().unwrap();

        let mut rtc = RtcExt::constrain(device.RTC0);
        rtc.set_prescaler(0xFFF).unwrap();
        rtc.enable_interrupt(RtcInterrupt::Tick);

        let mut alarm = Wecker::new(UnixTimestamp(1554041486));

        // CEST
        alarm.time.set_local_time_zone(FixedOffsetFromUtc::from_hours_and_minutes(2, 0));

        // alarm.alarms.push(Uhr::from(UnixTimestamp(1554041486 + 10))).unwrap();

        let mut next_alarm = Uhr::from(UnixTimestamp(1554041486 + 10));
        next_alarm.set_local_time_zone(FixedOffsetFromUtc::from_hours_and_minutes(2, 0));

        alarm.insert_alarm(next_alarm, DayFlags::SUNDAY).unwrap();
        // alarm.alarms.push(Uhr::from(UnixTimestamp(1554041486 + 25))).unwrap();
        // alarm.alarms.push(Uhr::from(UnixTimestamp(1554041486 + 20))).unwrap();
        // alarm.alarms.push(Uhr::from(UnixTimestamp(1554041486 + 10))).unwrap();

        RTCT = rtc.enable_counter();
        RANDOM = rng;
        DW_RST_PIN = rst_pin;
        DW1000 = dw1000;
        LOGGER = Logger::new(uarte0);
        TIMER = timer;
        LED_RED_1 = pins.p0_14.degrade().into_push_pull_output(Level::High);
        ALARM_CLOCK = alarm;
    }

    #[idle(resources = [TIMER, RANDOM, DW1000])]
    fn idle() -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[interrupt(resources = [ALARM_CLOCK, RTCT, LED_RED_1, LOGGER])]
    fn RTC0() {
        static mut TOGG: bool = false;
        static mut STEP: u32 = 0;
        const TICK_TIME: &Duration = &Duration::from_millis(125);

        (*resources.RTCT).get_event_triggered(RtcInterrupt::Tick, true);

        if *TOGG {
            (*resources.LED_RED_1).set_low();
        } else {
            (*resources.LED_RED_1).set_high();
        }

        let mut out: String<U1024> = String::new();


        resources.ALARM_CLOCK.time.increment(TICK_TIME);
        if resources.ALARM_CLOCK.alarm_ready() {
            out.clear();
            write!(&mut out, "!!! ALARM !!!").unwrap();
            (*resources.LOGGER).error(&out).unwrap();
            out.clear();
            write!(&mut out, "{:?}", resources.ALARM_CLOCK).unwrap();
            (*resources.LOGGER).log(&out).unwrap();
        }

        if (*STEP & 0x7) == 0 {
            let time = resources.ALARM_CLOCK.time.into_local_date_time();
            out.clear();
            write!(
                &mut out,
                "TIME {:02}:{:02}:{:02}",
                time.hour(),
                time.minute(),
                time.second(),
            ).unwrap();
            (*resources.LOGGER).log(&out).unwrap();
        }

        *STEP += 1;
        *TOGG = !*TOGG;
    }
};
