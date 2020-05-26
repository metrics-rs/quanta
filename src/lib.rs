//! Performant cross-platform timing with goodies.
//!
//! `quanta` provides a simple and fast API for measuring the current time and the duration between
//! events.  It does this by providing a thin layer on top of native OS timing functions, or, if
//! available, using the Time Stamp Counter feature found on modern CPUs.
//!
//! # Design
//! Internally, `quanta` maintains the concept of two potential clock sources: a reference clock and
//! a source clock.
//!
//! The reference clock is provided by the OS, and always available.  It is equivalent to what is
//! provided by the standard library in terms of the underlying system calls being made.  As it
//! uses the native timing facilities provided by the operating system, we ultimately depend on the
//! OS itself to give us a stable and correct value.
//!
//! The source clock is a potential clock source based on the Time Stamp Counter feature found on
//! modern CPUs.  If the TSC feature is not present or is not reliable enough, `quanta` will
//! transparently utilize the reference clock instead.
//!
//! Depending on the underlying processor(s) in the system, `quanta` will figure out the most
//! accurate/efficient way to calibrate the source clock to the reference clock in order to provide
//! measurements scaled to wall clock time.
//!
//! Details on TSC support, and calibration, are detailed below.
//!
//! # Features
//! Beyond simply taking measurements of the current time, `quanta` provides features for more easily
//! working with clocks, as well as being able to enhance performance further:
//! - `Clock` can be mocked for testing
//! - globally accessible "recent" time with amortized overhead
//!
//! For any code that uses a `Clock`, a mocked version can be substituted.  This allows for
//! application authors to control the time in tests, which allows simulating not only the normal
//! passage of time but provides the ability to warp time forwards and backwards in order to test
//! corner cases in logic, etc.  Creating a mocked clock can be acheived with [`Clock::mock`], and
//! [`Mock`] contains more details on mock usage.
//!
//! `quanta` also provides a "recent" time feature, which allows a slightly-delayed version of time
//! to be provided to callers, trading accuracy for speed of access.  An upkeep thread is spawned,
//! which is responsible for taking measurements and updating the global recent time. Callers then
//! can access the cached value by calling `Clock::recent`.  This interface can be 4-10x faster
//! than directly calling `Clock::now`, even when TSC support is available.  As the upkeep thread
//! is the only code updating the recent time, the accuracy of the value given to callers is
//! limited by how often the upkeep thread updates the time, thus the trade off between accuracy
//! and speed of access.
//!
//! # Platform Support
//! At a high level, `quanta` carries support for most major operating systems out of the box:
//! - Windows ([QueryPerformanceCounter])
//! - macOS/OS X/iOS ([mach_continuous_time])
//! - Linux/*BSD/Solaris ([clock_gettime])
//!
//! These platforms are supported in the "reference" clock sense, and support for using the Time
//! Stamp Counter as a clocksource is more subtle, and explained below.
//!
//! # Time Stamp Counter support
//! Accessing the TSC requires being on the x86_64 architecture, with access to SSE2. Additionally,
//! the processor must support either constant or nonstop/invariant TSC.  This ensures that the TSC
//! ticks at a constant rate which can be easily scaled.
//!
//! A caveat is that "constant" TSC doesn't account for all possible power states (levels of power
//! down or sleep that a CPU can enter to save power under light load, etc) and so a constant TSC
//! can lead to drift in measurements over time, after they've been scaled to reference time.
//!
//! This is a limitation of the TSC mode, as well as the nature of `quanta` not being able to know,
//! as the OS would, when a power state transition has happened, and thus compensate with a
//! recalibration. Nonstop/invariant TSC does not have this limitation and is stable over long
//! periods of time.
//!
//! Roughly speaking, the following list contains the beginning model/generation of processors
//! where you should be able to expect having invariant TSC support:
//! - Intel Nehalem and newer for server-grade
//! - Intel Skylake and newer for desktop-grade
//! - VIA Centaur Nano and newer (circumstantial evidence here)
//! - AMD Phenom and newer
//!
//! Ultimately, `quanta` will query CPUID information to determine if the processor has the
//! required features to use the TSC.
//!
//! # Calibration
//! As the TSC doesn't necessarily tick at reference scale -- i.e. one tick isn't always one
//! nanosecond -- we have to apply a scaling factor when converting from source to reference time
//! scale.  We acquire this scaling factor by querying the processor or calibrating our source
//! clock to the reference clock.
//!
//! In some cases, on newer processors, the frequency of the TSC can be queried directly, providing
//! a fixed scaling factor with no further calibration necessary.  In other cases, `quanta` will
//! have to run its own calibration before the clock is ready to be used: repeatedly taking
//! measurements from both the reference and source clocks until a stable scaling factor has been
//! established.
//!
//! This calibration is stored globally and reused.  However, the first `Clock` that is created in
//! an application will block for a small period of time as it runs this calibration loop.  The
//! time spent in the calibration loop is limited to 200ms overall.  In practice, `quanta` will
//! reach a stable calibration quickly (usually 10-20ms, if not less) and so this deadline is
//! unlikely to be reached.
//!
//! # Caveats
//! Utilizing the TSC can be a tricky affair, and so here is a list of caveats that may or may not
//! apply, and is in no way exhaustive:
//! - CPU hotplug behavior is undefined
//! - raw values may time warp
//! - measurements from the TSC may drift past or behind the comparable reference clock
//!
//! [QueryPerformanceCounter]: https://msdn.microsoft.com/en-us/library/ms644904(v=VS.85).aspx
//! [mach_continuous_time]: https://developer.apple.com/documentation/kernel/1646199-mach_continuous_time
//! [clock_gettime]: https://linux.die.net/man/3/clock_gettime
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use once_cell::sync::OnceCell;

mod monotonic;
use self::monotonic::Monotonic;
mod counter;
use self::counter::Counter;
mod mock;
pub use self::mock::{IntoNanoseconds, Mock};
mod instant;
pub use self::instant::Instant;
mod upkeep;
pub use self::upkeep::{Handle, Upkeep};
mod stats;
use self::stats::Variance;

static GLOBAL_RECENT: AtomicU64 = AtomicU64::new(0);
static GLOBAL_CALIBRATION: OnceCell<Calibration> = OnceCell::new();

// Run 100 rounds of calibration before we start actually seeing what the numbers look like.
const MINIMUM_CAL_ROUNDS: u64 = 500;

// We want our maximum error to be 10 nanoseconds.
const MAXIMUM_CAL_ERROR_NS: u64 = 10;

// Don't run the calibration loop for longer than 200ms of wall time.
const MAXIMUM_CAL_TIME: Duration = Duration::from_millis(200);

const BASE_LEAVES: u32 = 0x0;
const EXTENDED_LEAVES: u32 = 0x8000_0000;
const APMI_LEAF: u32 = 0x8000_0007;

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum ClockType {
    Monotonic(Monotonic),
    Counter(u64, Monotonic, Counter, Calibration),
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

#[derive(Debug, Copy, Clone)]
pub(crate) struct Calibration {
    ref_time: u64,
    src_time: u64,
    scale_factor: u64,
    scale_shift: u32,
}

impl Calibration {
    pub fn new() -> Calibration {
        Calibration {
            ref_time: 0,
            src_time: 0,
            scale_factor: 1,
            scale_shift: 1,
        }
    }

    fn calibrate<R, S>(&mut self, reference: &R, source: &S)
    where
        R: ClockSource,
        S: ClockSource,
    {
        let mut variance = Variance::default();
        let deadline = reference.now() + MAXIMUM_CAL_TIME.as_nanos() as u64;

        self.ref_time = reference.now();
        self.src_time = source.now();

        println!(
            "fancy2: start={} deadline={} ({})",
            self.ref_time,
            deadline,
            MAXIMUM_CAL_TIME.as_nanos()
        );

        let loop_delta = 1000;
        loop {
            // Busy loop to burn some time.
            let mut last = reference.now();
            let target = last + loop_delta;
            while last < target {
                last = reference.now();
            }

            // We put an upper bound on how long we run calibration before to provide a predictable
            // overhead to the calibration process.  In practice, even if we hit the calibration
            // deadline, we should still have run a sufficient number of rounds to get an accurate
            // calibration.
            if last >= deadline {
                println!("fancy2: hit cal loop deadline");

                let mean = variance.mean().abs();
                let mean_error = variance.mean_error().abs();
                println!(
                    "fancy2: stats at deadline: mean={} error={}, max_error={}",
                    mean, mean_error, MAXIMUM_CAL_ERROR_NS
                );
                break;
            }

            // Adjust our calibration before we take our measurement.
            self.adjust_cal_ratio(reference, source);

            let r_time = reference.now();
            let s_raw = source.now();
            let s_time = scale_src_to_ref(s_raw, &self);
            variance.add(s_time as f64 - r_time as f64);

            if variance.has_significant_result() {
                let mean = variance.mean().abs();
                let mean_error = variance.mean_error().abs();
                let mean_with_error = variance.mean_with_error();
                let samples = variance.samples();

                if samples > MINIMUM_CAL_ROUNDS
                    && mean_with_error < MAXIMUM_CAL_ERROR_NS
                    && mean_error / mean <= 1.0
                {
                    //&& self.scale_factor < 8000 {
                    println!(
                        "fancy2: reached error threshold (mean={} error={}, max_error={})",
                        mean, mean_error, MAXIMUM_CAL_ERROR_NS
                    );
                    println!(
                        "fancy2: cal config: num={} denom={}",
                        self.scale_factor, self.scale_shift
                    );
                    break;
                }
            }
        }

        let delta = reference.now() - self.ref_time;
        println!("fancy2: completed in {}ns", delta);
    }

    fn adjust_cal_ratio<R, S>(&mut self, reference: &R, source: &S)
    where
        R: ClockSource,
        S: ClockSource,
    {
        // Overall algorithm: measure the reference and source clock deltas, which leaves us witrh
        // a fraction of ref_d/src_d representing our source-to-reference clock scaling ratio.
        //
        // Find the next highest number, starting with src_d, that is a power of two.  That number
        // is now our new denominator in the scaling ratio.  We scale the old numerator (ref_d) by
        // a commensurate amount to match the delta between src_d and src_d_po2.
        let ref_end = reference.now();
        let src_end = source.end();

        let ref_d = ref_end.wrapping_sub(self.ref_time);
        let src_d = src_end.wrapping_sub(self.src_time);

        // TODO: we should almost never get a zero here because that would mean denom was greater
        // than 2^63 which is already a red flag.. but i'm not 100% sure if we can prove it well
        // enough to simply keep the panic around? gotta think on this
        let src_d_po2 = src_d.next_power_of_two();
        if src_d_po2 == 0 {
            panic!("po2_denom was zero!");
        }

        // TODO: lossy conversion back and forth just to get an approximate value, can we do better
        // with integer math? not sure
        let po2_ratio = src_d_po2 as f64 / src_d as f64;
        self.scale_factor = (ref_d as f64 * po2_ratio) as u64;
        self.scale_shift = src_d_po2.trailing_zeros();
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
    /// Creates a new clock with the optimal reference and source clocks.
    ///
    /// Support for TSC, etc, are checked at the time of creation, not compile-time.
    pub fn new() -> Clock {
        let reference = Monotonic::new();
        let constant_tsc = unsafe { has_constant_or_better_tsc() };
        let inner = if constant_tsc {
            let source = Counter::new();
            let calibration = GLOBAL_CALIBRATION.get_or_init(|| {
                let mut calibration = Calibration::new();
                calibration.calibrate(&reference, &source);
                calibration
            });
            ClockType::Counter(0, reference, source, *calibration)
        } else {
            ClockType::Monotonic(reference)
        };

        Clock { inner }
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
    /// This method is the spiritual equivalent of [`std::time::Instant::now`].  It is guaranteed
    /// to return a monotonically increasing value between calls to the same `Clock` instance.
    ///
    /// Returns an [`Instant`].
    pub fn now(&mut self) -> Instant {
        match &self.inner {
            ClockType::Monotonic(monotonic) => Instant(monotonic.now()),
            ClockType::Counter(mut last, _, counter, _) => {
                let now = counter.now();
                if now > last {
                    last = now;
                }
                self.scaled(last)
            }
            ClockType::Mock(mock) => Instant(mock.now()),
        }
    }

    /// Gets the underlying time from the fastest available clock source.
    ///
    /// As the clock source may or may not be the TSC, value is not guaranteed to be in nanoseconds
    /// or to be monotonic.  Value can be scaled to reference time by calling either [`scaled`]
    /// or [`delta`].
    ///
    /// If you need maximum accuracy in your measurements, consider using [`start`] and [`end`].
    ///
    /// [`scaled`]: Clock::scaled
    /// [`delta`]: Clock::delta
    /// [`start`]: Clock::start
    /// [`end`]: Clock::end
    pub fn raw(&self) -> u64 {
        match &self.inner {
            ClockType::Monotonic(monotonic) => monotonic.now(),
            ClockType::Counter(_, _, counter, _) => counter.now(),
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
            ClockType::Monotonic(monotonic) => monotonic.start(),
            ClockType::Counter(_, _, counter, _) => counter.start(),
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
            ClockType::Monotonic(monotonic) => monotonic.end(),
            ClockType::Counter(_, _, counter, _) => counter.end(),
            ClockType::Mock(mock) => mock.now(),
        }
    }

    /// Scales a raw measurement to reference time.
    ///
    /// You must scale raw measurements to ensure your result is in nanoseconds.  The raw
    /// measurement is not guaranteed to be in nanoseconds and may vary.  It is only OK to avoid
    /// scaling raw measurements if you don't need actual nanoseconds.
    ///
    /// Returns an [`Instant`].
    pub fn scaled(&self, value: u64) -> Instant {
        match &self.inner {
            ClockType::Counter(_, _, _, calibration) => {
                Instant(scale_src_to_ref(value, &calibration) as u64)
            }
            _ => Instant(value),
        }
    }

    /// Calculates the delta between two measurements, and scales to reference time.
    ///
    /// This method is slightly faster when you know you need the delta between two raw
    /// measurements, or a start/end measurement, than using [`scaled`] for both conversions.
    ///
    /// [`scaled`]: Clock::scaled
    pub fn delta(&self, start: u64, end: u64) -> Duration {
        // Safety: we want wrapping_sub on the end/start delta calculation so that two measurements
        // split across a rollover boundary still return the right result.  However, we also know
        // the TSC could potentially give us different values between cores/sockets, so we're just
        // doing our due diligence here to make sure we're not about to create some wacky duration.
        if end <= start {
            return Duration::new(0, 0);
        }

        let raw_delta = end.wrapping_sub(start);
        let scaled = match &self.inner {
            ClockType::Counter(_, _, _, calibration) => {
                mul_div_po2_u64(raw_delta, calibration.scale_factor, calibration.scale_shift)
            }
            _ => raw_delta,
        };
        Duration::from_nanos(scaled)
    }

    /// Gets the most recent current time, scaled to reference time.
    ///
    /// This method provides ultra-low-overhead access to a slightly-delayed version of the current
    /// time.  Instead of querying the underlying source clock directly, a shared, global value is
    /// read directly without the need to scale to reference time.
    ///
    /// The upkeep thread must be started in order to update the time.  You can read the
    /// documentation for [`Builder`] for more information on starting the upkeep thread, as well
    /// as the details of the "current time" mechanism.
    ///
    /// If the upkeep thread has not been started, the return value will be `0`.
    ///
    /// Returns an [`Instant`].
    pub fn recent(&self) -> Instant {
        match &self.inner {
            ClockType::Mock(mock) => Instant(mock.now()),
            _ => Instant(GLOBAL_RECENT.load(Ordering::Relaxed)),
        }
    }

    /// Updates the recent current time.
    pub(crate) fn upkeep(value: Instant) {
        GLOBAL_RECENT.store(value.0, Ordering::Release);
    }
}

impl Default for Clock {
    fn default() -> Clock {
        Clock::new()
    }
}

#[inline]
fn scale_src_to_ref(src_raw: u64, cal: &Calibration) -> u64 {
    let delta = src_raw.saturating_sub(cal.src_time);
    let scaled = mul_div_po2_u64(delta, cal.scale_factor, cal.scale_shift);
    scaled + cal.ref_time
}

#[inline]
fn mul_div_po2_u64(value: u64, numer: u64, denom: u32) -> u64 {
    // Modified muldiv routine where the denominator has to be a power of two. `denom` is expected
    // to be the number of bits to shift, not the actual decimal value.
    let mut v: u128 = value as u128;
    v *= numer as u128;
    v >>= denom;
    v as u64
}

#[allow(dead_code)]
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
unsafe fn has_constant_or_better_tsc() -> bool {
    // All of this special handling for CPU manufacturers, specific family/model combinations, etc,
    // is derived from the Linux kernel, master branch, as of 2020-05-19.

    // We limit ourselves to Intel, AMD, and Centaur.  There are technically other CPUs that
    // seemingly support constant or better TSC, but there's no chance in hell I have access to
    // test that out.  People can reach out if they want support.
    let cpu_mfg = read_cpuid_mfg();
    match cpu_mfg.as_str() {
        "GenuineIntel" => {
            // All Intel processors from the Core microarchitecture and on.
            if read_cpuid_family_model() >= 0x60E {
                return true;
            }
        }
        "AuthenticAMD" => {}
        "CentaurHauls" => {
            // VIA Nano and above should at least have constant TSC.
            if read_cpuid_family_model() >= 0x60F {
                return true;
            }
        }
        // Unknown/unsupported processor.
        _ => return false,
    }

    // TODO: if vendor isn't intel, and num cpus (not cores) > 1, good chance we won't be synced
    // (kernel/tsc.c:1214) looks like lscpu can detect socket count, can we emulate it?
    if has_multiple_sockets() && cpu_mfg != "GenuineIntel" {
        return false;
    }

    // We check CPUID for nonstop/invariant TSC as our fallback. (CPUID EAX=0x8000_0007, bit 8)
    read_cpuid_nonstop_tsc()
}

#[allow(dead_code)]
#[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
unsafe fn has_constant_or_better_tsc() -> bool {
    false
}

unsafe fn read_cpuid_mfg() -> String {
    use core::arch::x86_64::__cpuid;

    let result = __cpuid(0);
    let mut buf = [0u8; 12];
    buf[0..4].copy_from_slice(&result.ebx.to_le_bytes()[..]);
    buf[4..8].copy_from_slice(&result.edx.to_le_bytes()[..]);
    buf[8..12].copy_from_slice(&result.ecx.to_le_bytes()[..]);

    String::from_utf8_unchecked(Vec::from(&buf[..]))
}

unsafe fn read_cpuid_nonstop_tsc() -> bool {
    use core::arch::x86_64::{__cpuid, __get_cpuid_max};

    // Make sure the given leaf we need to check exists. 0x8000_0007 is the "advanced power
    // management information" that contains whether or not nonstop TSC is supported.
    let (highest_leaf, _) = __get_cpuid_max(EXTENDED_LEAVES);
    if highest_leaf < APMI_LEAF {
        println!("cpuid level too low to read InvariantTSC: {}", highest_leaf);
        return false;
    }

    let result = __cpuid(APMI_LEAF);
    let nonstop = (result.edx & (1 << 8)) != 0;
    println!("nonstop tsc: {}", nonstop);
    nonstop
}

unsafe fn read_cpuid_intel_tsc_frequency() -> Option<u64> {
    use core::arch::x86_64::{__cpuid, __get_cpuid_max};

    // Only available on Intel.
    let cpu_mfg = read_cpuid_mfg();
    if cpu_mfg != "GenuineIntel" {
        println!("not intel");
        return None;
    }

    // Make sure the given leaf we need to check exists.
    let (highest_leaf, _) = __get_cpuid_max(BASE_LEAVES);
    if highest_leaf < 0x15 {
        println!("0x15 leaf not available");
        return None;
    }

    // Time Stamp Counter and Nominal Core Crystal Clock Information Leaf (0x15H)
    //
    // We need the numerator (EBX) and the denominator (EAX) to both be present otherwise we can't
    // do anything, but the core crystal frequency (ECX) may not be present.
    // it another way if it's not available here.
    let result = __cpuid(0x15);
    if result.ebx == 0 || result.eax == 0 {
        println!("no numerator/denominator available");
        return None;
    }

    let numerator = result.ebx as u64;
    let denominator = result.eax as u64;

    // If we didn't get the core crystal frequency from 0x15, try mapping it to known frequencies.
    // (Intel SDM, Vol. 3B, 18-137, section 18.7.3, table 18-85, "Nominal Core Crystal Clock Frequency")
    let mut crystal_hz = result.ecx as u64;
    if crystal_hz == 0 {
        let mapped_crystal_hz = match read_cpuid_family_model() {
            // Intel Xeon Processor Scalable Family (Skylake, Cascade Lake, Cooper Lake)
            0x655 => 25_000_000,
            // 6th generation/7th generation processors (Skylake, Kaby Lake)
            0x64E | 0x65E | 0x68E | 0x69E => 24_000_000,
            // 2nd, 3rd, 4th, and 5th generation processors (Sandy Bridge, Ivy Bridge, Haswell, Broadwell)
            0x62A | 0x63A | 0x63C | 0x645 | 0x646 | 0x647 | 0x63D | 0x62D | 0x63E | 0x63F
            | 0x656 | 0x64F => {
                // We multiply by the reciprocal of the core crystal clock ratio because we're
                // returning the TSC frequency directly, so when we go to calculate the ratio, we
                // want to cancel out the normal ratio math.
                if highest_leaf < 0x16 {
                    println!("0x16 leaf not available");
                    0
                } else {
                    0
                }
            }
            // Whatever it is, we don't handle it yet.
            _ => 0,
        };

        if mapped_crystal_hz != 0 {
            crystal_hz = mapped_crystal_hz;
        }
    }

    // We failed to get the crystal frequency or processor base frequency, so we can't calculate
    // the ratio.  Womp.  Bail out.
    if crystal_hz == 0 {
        return None;
    }

    Some(crystal_hz * (numerator / denominator))
}

unsafe fn read_cpuid_family_model() -> u32 {
    use core::arch::x86_64::__cpuid;

    // Intel doesn't use extended family as far as I can tell, so basically we compute the
    // family/model identifier as:
    //
    // Nobody seems to use extended family, so we munge together the following fields:
    //
    // [8 bits - family] [4 bits - extended model] [4 bits - model]
    //
    // Thus, for Intel Skylake (family=6, extended model=5, model=5), our return value would be
    // 0x0655
    let result = __cpuid(0x01);
    let family = result.eax & (0xF << 8);
    let extended_model = result.eax & (0xF << 16);
    let model = result.eax & (0xF << 4);

    family + extended_model + model
}

unsafe fn read_msr_intel_platform_info_bus_frequency() -> Option<u64> {
    // TODO: implement me! literally only doable on linux, though, via /dev/cpu, which requires
    // root so this may be a crapshoot in terms of providing a solid fallback for older CPUs
    None
}

fn has_multiple_sockets() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{Clock, ClockSource, Monotonic};
    use average::{Merge, Variance};

    #[test]
    fn test_mock() {
        let (mut clock, mock) = Clock::mock();
        assert_eq!(clock.now().as_u64(), 0);
        mock.increment(42);
        assert_eq!(clock.now().as_u64(), 42);
    }

    #[test]
    fn test_now() {
        let mut clock = Clock::new();
        assert!(clock.now().as_u64() > 0);
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
        let scaled = clock.scaled(raw);
        assert!(scaled.as_u64() > 0);
    }

    #[test]
    fn test_reference_source_calibration() {
        let mut clock = Clock::new();
        let reference = Monotonic::new();

        let loops = 10000;
        let samples = 1024;

        let mut overall = Variance::new();
        let mut deltas = Vec::with_capacity(samples);
        deltas.reserve(samples);

        for _ in 0..loops {
            deltas.clear();
            for _ in 0..samples {
                let qstart = clock.now();
                let rstart = reference.now();

                deltas.push(rstart.saturating_sub(qstart.as_u64()));
            }

            let local = deltas.iter().map(|i| *i as f64).collect::<Variance>();
            overall.merge(&local);
        }

        println!(
            "reference/source delta: mean={} error={}",
            overall.mean(),
            overall.error()
        );

        // If things are out of sync more than 1000ns, something is likely scaled wrong.
        assert!(overall.mean() < 1000.0);
    }

    #[test]
    fn test_reference_self_calibration() {
        let reference = Monotonic::new();

        let loops = 10000;
        let samples = 1024;

        let mut overall = Variance::new();
        let mut deltas = Vec::with_capacity(samples);
        deltas.reserve(samples);

        for _ in 0..loops {
            deltas.clear();
            for _ in 0..samples {
                let rstart = reference.now();
                let rend = reference.now();

                deltas.push(rend - rstart);
            }

            let local = deltas.iter().map(|i| *i as f64).collect::<Variance>();
            overall.merge(&local);
        }

        println!(
            "reference/reference inter-call delta: mean={} error={}",
            overall.mean(),
            overall.error()
        );

        // If things are out of sync more than 1000ns, then I dunno, because our reference is
        // supposed to be reliable. 😬
        assert!(overall.mean() < 1000.0);
    }
}