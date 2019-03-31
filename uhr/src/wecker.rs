use core::cmp::Ordering;
use core::convert::From;
use core::time::Duration;

use bitflags::bitflags;
use generic_array::ArrayLength;
use gregor::{DayOfTheWeek, UnixTimestamp};
use heapless::binary_heap::{BinaryHeap, Min};

use crate::uhr::Uhr;

bitflags! {
    /// A bit packed structure representing days of the week
    pub struct DayFlags: u8 {
        const MONDAY    = 0b0000_0001;
        const TUESDAY   = 0b0000_0010;
        const WEDNESDAY = 0b0000_0100;
        const THURSDAY  = 0b0000_1000;
        const FRIDAY    = 0b0001_0000;
        const SATURDAY  = 0b0010_0000;
        const SUNDAY    = 0b0100_0000;

        const WEEKDAYS = Self::MONDAY.bits |
            Self::TUESDAY.bits |
            Self::WEDNESDAY.bits |
            Self::THURSDAY.bits |
            Self::FRIDAY.bits;
        const WEEKENDS = Self::SATURDAY.bits | Self::SUNDAY.bits;
    }
}

impl From<DayOfTheWeek> for DayFlags {
    fn from(dow: DayOfTheWeek) -> DayFlags {
        match dow {
            DayOfTheWeek::Monday => DayFlags::MONDAY,
            DayOfTheWeek::Tuesday => DayFlags::TUESDAY,
            DayOfTheWeek::Wednesday => DayFlags::WEDNESDAY,
            DayOfTheWeek::Thursday => DayFlags::THURSDAY,
            DayOfTheWeek::Friday => DayFlags::FRIDAY,
            DayOfTheWeek::Saturday => DayFlags::SATURDAY,
            DayOfTheWeek::Sunday => DayFlags::SUNDAY,
        }
    }
}

impl DayFlags {
    fn days_after(&self, today: DayOfTheWeek) -> u32 {
        // We must have some recurring item to reschedule
        debug_assert!(!self.is_empty());

        // Render the current day of the week as a bit flag
        let cur = DayFlags::from(today);

        // Obtain (u8) versions of our flag structure
        let curb = cur.bits();
        let repb = self.bits();

        // Overlay next week with this week
        let cur2: u32 = u32::from(curb | repb) | (u32::from(repb) << 7);

        // Get rid of days this week that have already passed
        let cur3 = cur2 >> curb.trailing_zeros() + 1;

        // How many blank days until the next repeat?
        let cur4 = cur3.trailing_zeros() + 1;

        // Our alarm does not support intervals greater than a week
        debug_assert!(cur4 <= 7);
        debug_assert!(cur4 > 0);

        cur4
    }
}

/// An opaque structure representing an alarm that may or may not repeat periodically
#[derive(Debug, Eq, PartialEq)]
pub struct Alarm {
    next_time: Uhr,
    repeat: DayFlags,
}

impl Ord for Alarm {
    fn cmp(&self, other: &Alarm) -> Ordering {
        self.next_time.cmp(&other.next_time)
    }
}

impl PartialOrd for Alarm {
    fn partial_cmp(&self, other: &Alarm) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// The specified alarm does not occur on its' specified repeat date
    AlarmNotOnRepeat,

    /// No space remains to push alarm
    AlarmFull,
}

/// A structure for storing a wall clock with associated alarms. Alarms
/// are stored in a binary heap, in a "soonest first" order.
#[derive(Debug)]
pub struct Wecker<ALARMS>
where
    ALARMS: ArrayLength<Alarm>,
{
    pub time: Uhr,
    alarms: BinaryHeap<Alarm, ALARMS, Min>,
}

impl<ALARMS> From<Uhr> for Wecker<ALARMS>
where
    ALARMS: ArrayLength<Alarm>,
{
    fn from(clock: Uhr) -> Self {
        Wecker {
            time: clock,
            alarms: BinaryHeap::new(),
        }
    }
}

impl<ALARMS> Wecker<ALARMS>
where
    ALARMS: ArrayLength<Alarm>,
{
    /// Create a new wall clock in UTC time, with no alarms
    pub fn new(time: UnixTimestamp) -> Self {
        Wecker {
            time: Uhr::from(time),
            alarms: BinaryHeap::new(),
        }
    }

    pub fn insert_alarm(&mut self, first_time: Uhr, repeat: DayFlags) -> Result<(), Error> {
        // If repeats, verify that first instance is on a repeat day
        if !repeat.is_empty() {
            let ftdt = first_time.into_local_date_time();
            let good = DayFlags::from(ftdt.day_of_the_week()).intersects(repeat);
            if !good {
                return Err(Error::AlarmNotOnRepeat);
            }
        }

        self.alarms
            .push(Alarm {
                next_time: first_time,
                repeat,
            })
            .map_err(|_| Error::AlarmFull)
    }

    /// Process all pending alarms, including rescheduling. If
    /// one or more alarms were ready, this function returns `true`
    pub fn alarm_ready(&mut self) -> bool {
        let mut flag = false;

        while {
            self.alarms
                .peek()
                .map(|alarm| alarm.next_time <= self.time)
                .unwrap_or(false)
        } {
            const ONE_DAY: Duration = Duration::from_secs(24 * 60 * 60);
            const ONE_WEEK: Duration = Duration::from_secs(7 * 24 * 60 * 60);

            // We know there is an alarm ready
            let mut alarm = self.alarms.pop().unwrap();
            flag = true;

            // Alarm doesn't repeat? All done!
            if alarm.repeat.is_empty() {
                continue;
            }

            // How many days until the next alarm instance?
            let days_til = alarm
                .repeat
                .days_after(self.time.into_local_date_time().day_of_the_week());

            // Increment the alarm to the next period
            alarm.next_time.increment(&(days_til * ONE_DAY));

            // Just in case we lost a lot of time, bump the week until we are
            // actually in the future from now
            while alarm.next_time < self.time {
                alarm.next_time.increment(&ONE_WEEK);
            }

            // We know there is space left, because we just popped one
            self.alarms.push(alarm).unwrap();
        }

        flag
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn days_until_test() {
        assert_eq!(
            DayFlags::days_after(&DayFlags::FRIDAY, DayOfTheWeek::Thursday),
            1
        );

        assert_eq!(
            DayFlags::days_after(&DayFlags::THURSDAY, DayOfTheWeek::Friday),
            6
        );

        assert_eq!(
            DayFlags::days_after(&DayFlags::WEEKDAYS, DayOfTheWeek::Friday),
            3
        );

        assert_eq!(
            DayFlags::days_after(&DayFlags::WEEKENDS, DayOfTheWeek::Sunday),
            6
        );

        assert_eq!(
            DayFlags::days_after(&DayFlags::THURSDAY, DayOfTheWeek::Thursday),
            7
        );
    }
}
