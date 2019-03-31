use core::convert::From;
use core::time::Duration;

use generic_array::ArrayLength;
use gregor::UnixTimestamp;
use heapless::binary_heap::{BinaryHeap, Min};

use crate::uhr::Uhr;

/// A structure for storing a wall clock with associated alarms. Alarms
/// are stored in a binary heap, in a "soonest first" order.
#[derive(Debug)]
pub struct Winkel<ALARMS>
where
    ALARMS: ArrayLength<Uhr>,
{
    pub time: Uhr,
    pub alarms: BinaryHeap<Uhr, ALARMS, Min>,
}

impl<ALARMS> From<Uhr> for Winkel<ALARMS>
where
    ALARMS: ArrayLength<Uhr>,
{
    fn from(clock: Uhr) -> Self {
        Winkel {
            time: clock,
            alarms: BinaryHeap::new(),
        }
    }
}

impl<ALARMS> Winkel<ALARMS>
where
    ALARMS: ArrayLength<Uhr>,
{
    /// Create a new wall clock in UTC time, with no alarms
    pub fn new(time: UnixTimestamp) -> Self {
        Winkel {
            time: Uhr::from(time),
            alarms: BinaryHeap::new(),
        }
    }

    /// Process all pending alarms, including rescheduling. If
    /// one or more alarms were ready, this function returns `true`.
    ///
    /// Currently all alarms are rescheduled to 24 hours after their last
    /// trigger time. This is likely to change in future releases
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
                alarm.increment(&Duration::from_secs(60 * 60 * 24));
            }

            // We know there is space left, because we just popped one
            self.alarms.push(alarm).unwrap();
            flag = true;
        }

        flag
    }
}
