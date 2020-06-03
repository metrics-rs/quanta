#[cfg(feature = "metrics")]
use metrics_core::AsNanoseconds;

use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration;

/// A point-in-time wall-clock measurement.
///
/// Represents a time measurement that has been taken by [`Clock`](crate::Clock) and scaled to reference time,
/// which is relative to the Unix epoch of 1970-01-01T00:00:00Z.
///
/// Unlike the stdlib `Instant`, this type has a meaningful difference: it is intended to be opaque, but the
/// internal value _can_ be accessed.  There are no guarantees here and depending on this value directly is
/// proceeding at your own risk. ⚠️
///
/// An `Instant` is 8 bytes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Instant(pub(crate) u64);

impl Instant {
    /// Returns the amount of time elapsed from another instant to this one.
    ///
    /// # Panics
    ///
    /// This function will panic if `earlier` is later than `self`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quanta::Clock;
    /// use std::time::Duration;
    /// use std::thread::sleep;
    ///
    /// let mut clock = Clock::new();
    /// let now = clock.now();
    /// sleep(Duration::new(1, 0));
    /// let new_now = clock.now();
    /// println!("{:?}", new_now.duration_since(now));
    /// ```
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.0
            .checked_sub(earlier.0)
            .map(Duration::from_nanos)
            .expect("supplied instant is later than self")
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or `None` if that instant is earlier than this one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quanta::Clock;
    /// use std::time::Duration;
    /// use std::thread::sleep;
    ///
    /// let mut clock = Clock::new();
    /// let now = clock.now();
    /// sleep(Duration::new(1, 0));
    /// let new_now = clock.now();
    /// println!("{:?}", new_now.checked_duration_since(now));
    /// println!("{:?}", now.checked_duration_since(new_now)); // None
    /// ```
    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        self.0.checked_sub(earlier.0).map(Duration::from_nanos)
    }

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is earlier than this one.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quanta::Clock;
    /// use std::time::Duration;
    /// use std::thread::sleep;
    ///
    /// let mut clock = Clock::new();
    /// let now = clock.now();
    /// sleep(Duration::new(1, 0));
    /// let new_now = clock.now();
    /// println!("{:?}", new_now.saturating_duration_since(now));
    /// println!("{:?}", now.saturating_duration_since(new_now)); // 0ns
    /// ```
    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        self.checked_duration_since(earlier)
            .unwrap_or_else(|| Duration::new(0, 0))
    }

    /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be represented as
    /// `Instant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.0.checked_add(duration.as_nanos() as u64).map(Instant)
    }

    /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be represented as
    /// `Instant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        self.0.checked_sub(duration.as_nanos() as u64).map(Instant)
    }

    /// Gets the inner value of this `Instant`.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    /// # Panics
    ///
    /// This function may panic if the resulting point in time cannot be represented by the
    /// underlying data structure. See [`Instant::checked_add`] for a version without panic.
    fn add(self, other: Duration) -> Instant {
        self.checked_add(other)
            .expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, other: Duration) {
        // This is not millenium-safe, but, I think that's OK. :)
        self.0 = self.0 + other.as_nanos() as u64;
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    fn sub(self, other: Duration) -> Instant {
        self.checked_sub(other)
            .expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, other: Duration) {
        // This is not millenium-safe, but, I think that's OK. :)
        self.0 = self.0 - other.as_nanos() as u64;
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        self.duration_since(other)
    }
}

impl PartialOrd for Instant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Instant {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl fmt::Debug for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "metrics")]
impl AsNanoseconds for Instant {
    fn as_nanos(&self) -> u64 {
        self.0
    }
}

#[cfg(feature = "prost")]
impl Into<prost_types::Timestamp> for Instant {
    fn into(self) -> prost_types::Timestamp {
        let dur = Duration::from_nanos(self.0);
        let secs = if dur.as_secs() > i64::MAX as u64 {
            i64::MAX
        } else {
            dur.as_secs() as i64
        };
        let nsecs = if dur.subsec_nanos() > i32::MAX as u32 {
            i32::MAX
        } else {
            dur.subsec_nanos() as i32
        };
        prost_types::Timestamp {
            seconds: secs,
            nanos: nsecs,
        }
    }
}