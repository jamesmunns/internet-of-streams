use core::cmp::{Ord, Ordering};
use core::convert::From;
use core::time::Duration;

use gregor::{DateTime, FixedOffsetFromUtc, UnixTimestamp};

/// A clock representing wall-clock-time. Not guaranteed to be
/// monotonic. Time is stored referenced to epoch/UTC time, and a
/// time zone offset may be provided to determine local time
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct Uhr {
    tz_offset: FixedOffsetFromUtc,
    seconds: UnixTimestamp,
    nanos: u32,
}

impl From<UnixTimestamp> for Uhr {
    fn from(uts: UnixTimestamp) -> Self {
        Uhr {
            seconds: uts,
            nanos: 0,
            tz_offset: FixedOffsetFromUtc::from_hours_and_minutes(0, 0),
        }
    }
}

impl Ord for Uhr {
    fn cmp(&self, other: &Uhr) -> Ordering {
        let sec_ord = self.seconds.0.cmp(&other.seconds.0);

        match sec_ord {
            Ordering::Equal => self.nanos.cmp(&other.nanos),
            order => order,
        }
    }
}

impl PartialOrd for Uhr {
    fn partial_cmp(&self, other: &Uhr) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Uhr {
    /// Increment the current clock by a duration
    pub fn increment(&mut self, dur: &Duration) {
        self.seconds.0 += dur.as_secs() as i64; // TODO
        self.nanos += dur.subsec_nanos();
        if self.nanos >= 1_000_000_000 {
            self.nanos -= 1_000_000_000;
            self.seconds.0 += 1;
        }
    }

    /// Create a new clock time at a time `dur` after the current time
    pub fn incremented(&self, dur: &Duration) -> Uhr {
        let mut new_time = *self;
        new_time.increment(dur);
        new_time
    }

    /// Obtain the duration since a given time. An error is returned if
    /// the before time is after now
    pub fn try_duration_since(&self, before: &Uhr) -> Result<Duration, ()> {
        if before > self {
            return Err(());
        }

        let mut delta_sec = self.seconds.0 - before.seconds.0;
        let delta_nanos = if before.nanos > self.nanos {
            delta_sec -= 1;
            (self.nanos + 1_000_000_000) - before.nanos
        } else {
            self.nanos - before.nanos
        };

        Ok(Duration::new(delta_sec as u64, delta_nanos))
    }

    /// Obtain the duration since a given time. This function panics if `before`
    /// is after the current time. Consider using `try_duration_since()`
    pub fn duration_since(&self, before: &Uhr) -> Duration {
        self.try_duration_since(before).unwrap()
    }

    /// Convert the current wall clock into a local `DateTime` object
    pub fn into_local_date_time(&self) -> DateTime<FixedOffsetFromUtc> {
        DateTime::from_timestamp(self.seconds, self.tz_offset)
    }

    /// Change the local timezone of the clock
    pub fn set_local_time_zone(&mut self, offset: FixedOffsetFromUtc) {
        self.tz_offset = offset;
    }
}
