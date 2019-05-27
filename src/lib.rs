//! High-speed timing facility.
//!
//! quanta provides a generalized interface to native high-speed timing mechanisms on different
//! target platforms, including support for accessing the TSC (time stamp counter) directly from
//! the processor.
//!
//! # Design
//!
//! Internally, two clocks are used — a reference and a source — to provide high-speed access to
//! timing values, while allowing for conversion back to a reference timescale that matches the
//! underlying system.
//!
//! Calibration between the reference and source happens at initialization time of [`Clock`].
//!
//! # Platform support
//!
//! quanta supports using the native high-speed timing facilities on the following platforms:
//! - Windows ([QueryPerformanceCounter])
//! - macOS/OS X ([mach_absolute_time])
//! - Linux/*BSD/Solaris ([clock_gettime])
//!
//! # TSC support
//!
//! Additionally, quanta has `RDTSC`/`RDTSCP` support for querying the time stamp counter directly
//! from the processor.  Using this mode has some caveats:
//! - you need to be running nightly to have access to the `asm!` macro/feature ([#29722])
//! - your processor needs to be recent enough to support a stable TSC mode like invariant or
//! constant TSC ([source][tsc_support])
//!
//! This feature is only usable when compiled with the `asm` feature flag.
//!
//! Generally speaking, most modern operating systems will already be attempting to use the TSC on
//! your behalf, along with switching to another clocksource if they determine that the TSC is
//! unstable or providing unsuitable speed/accuracy.  The primary reason to use TSC directly is that
//! calling it from userspace is measurably faster — although the "slower" methods are still on
//! the order of of tens of nanoseconds — and can be useful for timing operations which themselves
//! are _extremely_ fast.
//!
//! If your operations run in the hundreds of nanoseconds or less range, or you're measuring in a
//! tight loop, using the TSC support could help you avoid the normal overhead which would otherwise
//! contribute to a large chunk of actual time spent and would otherwise consume valuable cycles.
//!
//! [QueryPerformanceCounter]: https://msdn.microsoft.com/en-us/library/ms644904(v=VS.85).aspx
//! [mach_absolute_time]: https://developer.apple.com/library/archive/qa/qa1398/_index.html
//! [clock_gettime]: https://linux.die.net/man/3/clock_gettime
//! [#29722]: https://github.com/rust-lang/rust/issues/29722
//! [tsc_support]: http://oliveryang.net/2015/09/pitfalls-of-TSC-usage/
#![cfg_attr(feature = "tsc", feature(asm))]

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

mod monotonic;
use self::monotonic::Monotonic;
mod counter;
#[allow(unused_imports)]
use self::counter::Counter;
mod mock;
pub use self::mock::{IntoNanoseconds, Mock};
mod upkeep;
pub use self::upkeep::{Builder, Handle};

static GLOBAL_RECENT: AtomicU64 = AtomicU64::new(0);

type Reference = Monotonic;

#[cfg(feature = "tsc")]
type Source = Counter;
#[cfg(not(feature = "tsc"))]
type Source = Monotonic;

#[derive(Debug, Clone)]
enum ClockType {
    Optimized(Reference, Source, Calibration),
    Mock(Arc<Mock>),
}

/// A clock source that can provide the current time.
trait ClockSource {
    /// Gets the current time.
    ///
    /// Ideally, this method should return the time in nanoseconds to match the default clock, but
    /// it is not a requirement.
    fn now(&self) -> u64;

    /// Gets the current time, optimized for measuring the time before the start of a code section.
    ///
    /// This allows clock sources to provide a start-specific measurement that has a more accuracy
    /// than the general performance of [`now`].  This is useful for short sections of code where
    /// the reordering of CPU instructions could affect the actual start/end measurements.
    ///
    /// Ideally, this method should return the time in nanoseconds to match the default clock, but
    /// it is not a requirement.
    fn start(&self) -> u64;

    /// Gets the current time, optimized for measuring the time after the end of a code section.
    ///
    /// This allows clock sources to provide a end-specific measurement that has a more accuracy
    /// than the general performance of [`now`].  This is useful for short sections of code where
    /// the reordering of CPU instructions could affect the actual start/end measurements.
    ///
    /// Ideally, this method should return the time in nanoseconds to match the default clock, but
    /// it is not a requirement.
    fn end(&self) -> u64;
}

#[derive(Debug, Clone)]
pub(crate) struct Calibration {
    identical: bool,
    ref_time: f64,
    src_time: f64,
    ref_hz: f64,
    src_hz: f64,
    hz_ratio: f64,
}

impl Calibration {
    pub fn new() -> Calibration {
        Calibration {
            identical: false,
            ref_time: 0.0,
            src_time: 0.0,
            ref_hz: 1_000_000_000.0,
            src_hz: 1_000_000_000.0,
            hz_ratio: 1.0,
        }
    }

    #[allow(dead_code)]
    pub fn identical() -> Calibration {
        let mut calibration = Self::new();
        calibration.identical = true;
        calibration
    }

    #[allow(dead_code)]
    pub fn calibrate<R, S>(&mut self, reference: &R, source: &S)
    where
        R: ClockSource,
        S: ClockSource,
    {
        self.ref_time = reference.now() as f64;
        self.src_time = source.start() as f64;

        let ref_end = self.ref_time + self.ref_hz;

        loop {
            let t = reference.now() as f64;
            if t >= ref_end {
                break;
            }
        }

        let src_end = source.end() as f64;

        let ref_d = ref_end - self.ref_time;
        let src_d = src_end - self.src_time;

        self.src_hz = (src_d * self.ref_hz) / ref_d;
        self.hz_ratio = self.ref_hz / self.src_hz;
    }
}

impl Default for Calibration {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified clock for taking measurements.
#[derive(Debug, Clone)]
pub struct Clock {
    inner: ClockType,
}

impl Clock {
    /// Creates a new clock with the optimal reference and source.
    ///
    /// Both the reference clock and source clock are chosen at compile-time to be the fastest
    /// underlying clocks available.  The source clock is calibrated against the reference clock if
    /// need be.
    #[cfg(feature = "tsc")]
    pub fn new() -> Clock {
        let reference = Reference::new();
        let source = Source::new();
        let mut calibration = Calibration::new();
        calibration.calibrate(&reference, &source);

        Clock {
            inner: ClockType::Optimized(reference, source, calibration),
        }
    }

    /// Creates a new clock with the optimal reference and source.
    ///
    /// Both the reference clock and source clock are chosen at compile-time to be the fastest
    /// underlying clocks available.  The source clock is calibrated against the reference clock if
    /// need be.
    #[cfg(not(feature = "tsc"))]
    pub fn new() -> Clock {
        let reference = Reference::new();
        let source = Source::new();
        let calibration = Calibration::identical();

        Clock {
            inner: ClockType::Optimized(reference, source, calibration),
        }
    }

    /// Creates a new clock that is mocked for controlling the underlying time.
    ///
    /// Returns a [`Clock`] instance and a handle to the underlying [`Mock`] source so that the
    /// caller can control the passage of time.
    pub fn mock() -> (Clock, Arc<Mock>) {
        let mock = Arc::new(Mock::new());
        let clock = Clock {
            inner: ClockType::Mock(mock.clone()),
        };

        (clock, mock)
    }

    /// Gets the current time, scaled to reference time.
    ///
    /// Value is in nanoseconds.
    pub fn now(&self) -> u64 {
        match &self.inner {
            ClockType::Optimized(_, source, _) => self.scaled(source.now()),
            ClockType::Mock(mock) => mock.now(),
        }
    }

    /// Gets the underlying time from the source clock.
    ///
    /// Value is not guaranteed to be in nanoseconds.
    ///
    /// It requires conversion to reference time, however, via [`scaled`] or [`delta`].
    ///
    /// If you need maximum accuracy in your measurements, consider using [`start`] and [`end`].
    ///
    /// [`scaled`]: Clock::scaled
    /// [`delta`]: Clock::delta
    /// [`start`]: Clock::start
    /// [`end`]: Clock::end
    pub fn raw(&self) -> u64 {
        match &self.inner {
            ClockType::Optimized(_, source, _) => source.now(),
            ClockType::Mock(mock) => mock.now(),
        }
    }

    /// Gets the underlying time from the source clock, specific to starting an operation.
    ///
    /// Value is not guaranteed to be in nanoseconds.
    ///
    /// Provides the same functionality as [`raw`], but tries to ensure that no extra CPU
    /// instructions end up executing after the measurement is taken.  Since normal processors are
    /// typically out-of-order, other operations that logically come before a call to this method
    /// could be reordered to come after the measurement, thereby skewing the overall time
    /// measured.
    ///
    /// [`raw`]: Clock::raw
    pub fn start(&self) -> u64 {
        match &self.inner {
            ClockType::Optimized(_, source, _) => source.start(),
            ClockType::Mock(mock) => mock.now(),
        }
    }

    /// Gets the underlying time from the source clock, specific to ending an operation.
    ///
    /// Value is not guaranteed to be in nanoseconds.
    ///
    /// Provides the same functionality as [`raw`], but tries to ensure that no extra CPU
    /// instructions end up executing before the measurement is taken.  Since normal processors are
    /// typically out-of-order, other operations that logically come after a call to this method
    /// could be reordered to come before the measurement, thereby skewing the overall time
    /// measured.
    ///
    /// [`raw`]: Clock::raw
    pub fn end(&self) -> u64 {
        match &self.inner {
            ClockType::Optimized(_, source, _) => source.end(),
            ClockType::Mock(mock) => mock.now(),
        }
    }

    /// Scales a raw measurement to reference time.
    ///
    /// You must scale raw measurements to ensure your result is in nanoseconds.  The raw
    /// measurement is not guaranteed to be in nanoseconds and may vary.  It is only OK to avoid
    /// scaling raw measurements if you don't need actual nanoseconds.
    ///
    /// Value is in nanoseconds.
    pub fn scaled(&self, value: u64) -> u64 {
        match &self.inner {
            ClockType::Optimized(_, _, calibration) => {
                if calibration.identical {
                    value
                } else {
                    (((value as f64 - calibration.src_time) * calibration.hz_ratio) + calibration.ref_time) as u64
                }
            }
            ClockType::Mock(_) => value,
        }
    }

    /// Calculates the delta between two measurements, and scales to reference time.
    ///
    /// Value is in nanoseconds.
    ///
    /// This method is slightly faster when you know you need the delta between two raw
    /// measurements, or a start/end measurement, than using [`scaled`] for both conversions.
    ///
    /// [`scaled`]: Clock::scaled
    pub fn delta(&self, start: u64, end: u64) -> u64 {
        let raw_delta = end.wrapping_sub(start);
        match &self.inner {
            ClockType::Optimized(_, _, calibration) => (raw_delta as f64 * calibration.hz_ratio) as u64,
            ClockType::Mock(_) => raw_delta,
        }
    }

    /// Gets the most recent current time, scaled to reference time.
    ///
    /// This method provides ultra-low-overhead access to a slightly-delayed version of the current
    /// time.  Instead of querying the underlying source clock directly, an external task -- the
    /// upkeep thread -- is responsible for polling the time and updating a global reference to the
    /// "recent" time, which this method then loads.
    ///
    /// This method is usually at least 2x faster than querying the clock directly, and could be
    /// even faster in cases where getting the time goes through a heavy virtualized interface, or
    /// requires syscalls.
    ///
    /// The resolution of the time is still in nanoseconds, although the accuracy can only be as
    /// high as the interval at which the upkeep thread updates the global recent time.
    ///
    /// For example, the upkeep thread could be configured to update the time every millisecond,
    /// which would provide a measurement that should be, at most, 1ms behind the actual time.
    ///
    /// If the upkeep thread has not been started, the return value will be `0`.
    ///
    /// Value is in nanoseconds.
    pub fn recent(&self) -> u64 {
        match &self.inner {
            ClockType::Optimized(_, _, _) => GLOBAL_RECENT.load(Ordering::Acquire),
            ClockType::Mock(mock) => mock.now(),
        }
    }

    /// Updates the recent current time.
    pub(crate) fn upkeep(value: u64) {
        GLOBAL_RECENT.store(value, Ordering::Release);
    }
}

impl Default for Clock {
    fn default() -> Clock {
        Clock::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Clock;

    #[test]
    fn test_mock() {
        let (clock, mock) = Clock::mock();
        assert_eq!(clock.now(), 0);
        mock.increment(42);
        assert_eq!(clock.now(), 42);
    }

    #[test]
    fn test_now() {
        let clock = Clock::new();
        assert!(clock.now() > 0);
    }

    #[test]
    fn test_raw() {
        let clock = Clock::new();
        assert!(clock.raw() > 0);
    }

    #[test]
    fn test_start() {
        let clock = Clock::new();
        assert!(clock.start() > 0);
    }

    #[test]
    fn test_end() {
        let clock = Clock::new();
        assert!(clock.end() > 0);
    }

    #[test]
    fn test_scaled() {
        let clock = Clock::new();
        let raw = clock.raw();
        assert!(clock.scaled(raw) > 0);
    }
}
