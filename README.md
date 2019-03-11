# Internet of Streams Project

Building a wireless sensor platform from (almost) scratch in Embedded Rustlang. Built with a focus on teaching embedded systems, IoT development, and Rust through live streamed coding sessions. Devices based on the [DWM1001-DEV] board, the [Nordic nRF52] microcontroller, and [Embedded Rust].

[DWM1001-DEV]: https://www.decawave.com/product/dwm1001-development-board/
[Embedded Rust]: https://github.com/rust-embedded/wg
[Nordic nRF52]: https://www.nordicsemi.com/Products/Low-power-short-range-wireless/nRF52832

This project is sponsored by [Ferrous Systems](https://ferrous-systems.com). Interested in trainings or consulting on Embedded Systems, IoT projects, or the Rust Programming Language? [Get in touch!](mailto:iot-streams@ferrous-systems.com)

[![Ferrous Systems](https://ferrous-systems.com/images/ferrous-logo-text.svg)](https://ferrous-systems.com/)

## Stream Videos

You can find a [playlist of all videos here](https://www.youtube.com/playlist?list=PLX44HkctSkTewrL9frlUz0yeKLKecebT1) on YouTube.

* [2019-02-23] - **Hello Blinky World!**
    * Get the project set up
    * Get CI set up
    * Get HAL and RTFM set up
    * Blink the first LED
* [2019-02-28] - **COBS Encoding for Serial Framing**
    * Finish up a PR to get a streaming COBS encoder
* [2019-03-02] - **Async DMA UARTE - Part 1**
    * Get nrf52-hal vendored
    * Update some old code
    * Try to work around RTFM limitations
    * see the `uarte` branch for WIP
* [2019-03-07] - **Simple Blocking UART Logger**
    * Send data over the UART
    * Provide log/warn/err levels
    * Send data larger than a single 255 byte transaction
* [2019-03-10] - **Radio Work and `no_std` Serde**
    * Get messages sending periodically
    * Receive incoming messages
    * Use the hardware random number generator
    * Serialize/Deserialize messages with `ssmarshal` and `serde`

[2019-02-23]: https://youtu.be/S0VI70nY6Vo
[2019-02-28]: https://youtu.be/mnPbmPqKf1s
[2019-03-02]: https://youtu.be/O6KeMpnLRkI
[2019-03-07]: https://youtu.be/WYIei1MpVe4
[2019-03-10]: https://youtu.be/U2rC24XGtTk

## Future Topics

The following topics are planned to be addressed in future streams:

* Wireless Communication
* Bootloader/OTA updates
* Low Power Mode
* Logging
* Unit Testing
* Hardware in the Loop testing
* 6LoWPAN
* Bluetooth
* Gateway Router
* Messaging/Protocol/Serialization/Deserialization
* LED status codes

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
