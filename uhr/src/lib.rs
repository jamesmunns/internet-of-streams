#![cfg_attr(not(test), no_std)]

pub mod uhr;
pub mod wecker;

pub use crate::uhr::Uhr;
pub use crate::wecker::{Alarm, DayFlags, Wecker};
pub use generic_array::ArrayLength;
pub use gregor::{DateTime, FixedOffsetFromUtc, UnixTimestamp};
