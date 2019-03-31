#![cfg_attr(not(test), no_std)]

pub mod uhr;
pub mod wecker;

pub use crate::uhr::Uhr;
pub use crate::wecker::{
    Wecker,
    Alarm,
    DayFlags,
};
pub use gregor::{
    DateTime,
    FixedOffsetFromUtc,
    UnixTimestamp
};
pub use generic_array::ArrayLength;
