#![no_std]

use core::cmp::{Ord, Ordering};
use core::convert::From;
use core::time::Duration;

use generic_array::ArrayLength;
use gregor::{DateTime, FixedOffsetFromUtc, UnixTimestamp};
use heapless::binary_heap::{BinaryHeap, Min};

#[derive(Eq, PartialEq, Debug)]
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
    pub fn increment(&mut self, dur: Duration) {
        self.seconds.0 += dur.as_secs() as i64; // TODO
        self.nanos += dur.subsec_nanos();
        if self.nanos >= 1_000_000_000 {
            self.nanos -= 1_000_000_000;
            self.seconds.0 += 1;
        }
    }

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

    pub fn duration_since(&self, before: &Uhr) -> Duration {
        self.try_duration_since(before).unwrap()
    }

    pub fn into_local_date_time(&self) -> DateTime<FixedOffsetFromUtc> {
        DateTime::from_timestamp(
            self.seconds,
            self.tz_offset
        )
    }

    pub fn set_local_time_zone(&mut self, offset: FixedOffsetFromUtc) {
        self.tz_offset = offset;
    }
}

#[derive(Debug)]
pub struct Winkel<ALARMS>
where
    ALARMS: ArrayLength<Uhr>,
{
    pub time: Uhr,
    pub alarms: BinaryHeap<Uhr, ALARMS, Min>,
}

impl<ALARMS> Winkel<ALARMS>
where
    ALARMS: ArrayLength<Uhr>,
{
    pub fn new(time: UnixTimestamp) -> Self {
        Winkel {
            time: Uhr::from(time),
            alarms: BinaryHeap::new(),
        }
    }

    pub fn alarm_ready(&mut self) -> bool {
        let mut flag = false;

        while {
            self.alarms
                .peek()
                .map(|alarm| alarm <= &self.time)
                .unwrap_or(false)
        } {
            // We know there is an alarm ready
            let mut alarm = self.alarms.pop().unwrap();

            // In case we *really* missed the alarm
            // TODO: not a loop, actually figure out delta days
            while alarm < self.time {
                alarm.increment(Duration::from_secs(60 * 60 * 24));
            }

            // We know there is space left, because we just popped one
            self.alarms.push(alarm).unwrap();
            flag = true;
        }

        flag
    }
}
