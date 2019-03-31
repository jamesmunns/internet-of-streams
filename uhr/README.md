# `uhr` - A `no_std` wall clock

* [Documentation](https://docs.rs/uhr)

`uhr` aims to provide a generic abstraction for a time zone aware wall clock, as well as alarms associated with wall clock time.

It is intended to be used with a Real Time Counter, or any other "ticking" peripheral that can be used to increment a counter.

It is **NOT** a monotonic clock, and is not suitable as a replacement for `Instant`s and other similar structures. Time may move forward or backwards, due to time zone or daylight savings changes, or minor clock corrections provided by a more reliable source.

