use dwm1001::nrf52832_hal::{
    gpio::{
        p0::{
            // P0_19,
            P0_24,
        },
        Input,
        Floating,
        OpenDrainConfig,
        Level,
    },
};
use embedded_hal::blocking::delay::DelayMs;

/// The DW_RST pin (P0.24 on the nRF52)
///
/// Can be used to externally reset the DW1000.
#[allow(non_camel_case_types)]
pub struct DW_RST(Option<P0_24<Input<Floating>>>);

impl DW_RST {
    pub(crate) fn new<Mode>(p0_24: P0_24<Mode>) -> Self {
        DW_RST(Some(p0_24.into_floating_input()))
    }

    /// Externally reset the DW1000 using its RSTn pin
    ///
    /// The implementation of this method needs to wait a few times until the
    /// DW1000 is properly reset. To do that, it requires an implementation of
    /// [`DelayMs`] from the `embedded-hal` crate, which the user must supply.
    ///
    /// See [`nrf52832_hal::Delay`] for such an implementation.
    pub fn reset_dw1000<D>(&mut self, delay: &mut D) where D: DelayMs<u32> {
        // This whole `Option` thing is a bit of a hack. What we actually need
        // here is the ability to put the pin into a tri-state mode that allows
        // us to switch input/output on the fly.
        let dw_rst = self.0
            .take()
            .unwrap()
            // According the the DW1000 datasheet (section 5.6.3.1), the reset
            // pin should be pulled low using open-drain, and must never be
            // pulled high.
            .into_open_drain_output(
                OpenDrainConfig::Standard0Disconnect1,
                Level::Low
            );

        // Section 5.6.3.1 in the data sheet talks about keeping this low for
        // T-RST_OK, which would be 10-50 nanos. But table 15 makes it sound
        // like that should actually be T-DIG_ON (1.5-2 millis), which lines up
        // with the example code I looked at.
        delay.delay_ms(2);

        self.0 = Some(dw_rst.into_floating_input());

        // There must be some better way to determine whether the DW1000 is
        // ready, but I guess waiting for some time will do.
        delay.delay_ms(5);
    }
}


// /// The DW_IRQ pin (P0.19 on the nRF52)
// ///
// /// Can be used to wait for DW1000 interrupts.
// #[allow(non_camel_case_types)]
// pub struct DW_IRQ(P0_19<Input<Floating>>);

// impl DW_IRQ {
//     fn new<Mode>(p0_19: P0_19<Mode>) -> Self {
//         DW_IRQ(p0_19.into_floating_input())
//     }

//     /// Sets up DW1000 interrupt and goes to sleep until an interrupt occurs
//     ///
//     /// This method sets up the interrupt of the pin connected to DW_IRQ on the
//     /// DW1000 and goes to sleep, waiting for interrupts.
//     ///
//     /// There are two gotchas that must be kept in mind when using this method:
//     /// - This method returns on _any_ interrupt, even those unrelated to the
//     ///   DW1000.
//     /// - This method disables interrupt handlers. No interrupt handler will be
//     ///   called while this method is active.
//     pub fn wait_for_interrupts<T>(&mut self,
//         nvic:   &mut nrf52::NVIC,
//         gpiote: &mut nrf52::GPIOTE,
//         timer:  &mut Timer<T>,
//     )
//         where T: TimerExt
//     {
//         gpiote.config[0].write(|w| {
//             let w = w
//                 .mode().event()
//                 .polarity().lo_to_hi();

//             unsafe { w.psel().bits(19) }
//         });
//         gpiote.intenset.modify(|_, w| w.in0().set());

//         interrupt::free(|_| {
//             nrf52::NVIC::unpend(Interrupt::GPIOTE);
//             nrf52::NVIC::unpend(T::INTERRUPT);

//             nvic.enable(Interrupt::GPIOTE);
//             timer.enable_interrupt(nvic);

//             asm::dsb();
//             asm::wfi();

//             // If we don't do this, the (probably non-existing) interrupt
//             // handler will be called as soon as we exit this closure.
//             nvic.disable(Interrupt::GPIOTE);
//             timer.disable_interrupt(nvic);
//         });

//         gpiote.events_in[0].write(|w| unsafe { w.bits(0) });
//         gpiote.intenclr.modify(|_, w| w.in0().clear());
//     }
// }
