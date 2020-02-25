#[cfg(feature = "metrics")]
use metrics_core::AsNanoseconds;

use std::cmp::{Ord, PartialOrd, Ordering};
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::time::Duration;

/// A point-in-time wall-clock measurement.
///
/// Represents a time measurement that has been taken by [`Clock`](crate::Clock) and scaled to wall-clock time.
///
/// Unlike the stdlib `Instant`, this type has two meaningful differences:
/// - It provides no guarantees around monotonicity whatsoever, beyond any guarantees provided by
/// `Clock` itself.
/// - It is intended to be opaque, but the internal value can be accessed.  There are no guarantees
/// on the internal value timebase, or other factors, remaining stable over time and this
/// convenience is only intended for comparisons of `Instant`s provided by the same exact `Clock`
/// instance.
///
/// An `Instant` is 8 bytes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Instant(pub(crate) u64);

impl Instant {
    pub(crate) fn new(inner: u64) -> Self {
        Instant(inner)
    }

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
    /// let clock = Clock::new();
    /// let now = clock.now();
    /// sleep(Duration::new(1, 0));
    /// let new_now = clock.now();
    /// println!("{:?}", new_now.duration_since(now));
    /// ```
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.0.checked_sub(earlier.0)
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
    /// let clock = Clock::new();
    /// let now = clock.now();
    /// sleep(Duration::new(1, 0));
    /// let new_now = clock.now();
    /// println!("{:?}", new_now.checked_duration_since(now));
    /// println!("{:?}", now.checked_duration_since(new_now)); // None
    /// ```
    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        self.0.checked_sub(earlier.0)
            .map(Duration::from_nanos)
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
    /// let clock = Clock::new();
    /// let now = clock.now();
    /// sleep(Duration::new(1, 0));
    /// let new_now = clock.now();
    /// println!("{:?}", new_now.saturating_duration_since(now));
    /// println!("{:?}", now.saturating_duration_since(new_now)); // 0ns
    /// ```
    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        self.checked_duration_since(earlier)
            .unwrap_or(Duration::new(0, 0))
    }

    /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be represented as
    /// `Instant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.0.checked_add(duration.as_nanos() as u64)
            .map(Instant)
    }

    /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be represented as
    /// `Instant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        self.0.checked_sub(duration.as_nanos() as u64)
            .map(Instant)
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
