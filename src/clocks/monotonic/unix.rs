use std::mem;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default)]
pub struct Monotonic {
    _default: (),
}

impl Monotonic {
    #[allow(clippy::cast_sign_loss)]
    pub fn now(self) -> u64 {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe {
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
        }

        // LINT JUSTIFICATION:
        //
        // We really don't ever expect to actually _get_ negative values from `clock_gettime`, but
        // given the types, it's technically possible.  This is due to the fact that `tv_sec` is
        // supposed to be `time_t`, which Unix/POSIX-compliant systems implement as a signed number.
        // Accordingly, `tv_nsec` follows suit using a signed number.
        //
        // Given the adjustments made by NTP to clocks like CLOCK_MONOTONIC, and that
        // CLOCK_MONOTONIC can be anchored to an arbitrary point, and a whole skew of other
        // scenarios where it could be modified... it's technicaly possible to get back valid
        // negative values.  If we did math between `timespec` objects, the delta should be valid,
        // even with negative numbers.
        //
        // We're returning a u64 here, though, so it is what it is.  In practice, I've _never_ seen
        // negative values under normal operation.  If someone discovers a valid scenario where this
        // is violated and that we need to account for, I'll be colored impressed, but also, file an
        // issue and we'll do what we have to do to rework the code to try and support it better.
        //
        // Until then, though, we're just gonna ignore the lint.
        ts.tv_sec as u64 * 1_000_000_000 + ts.tv_nsec as u64
    }
}

// std::time::Instant is represented as
// struct Nanoseconds(u32);
// 
// struct Timespec {
//     tv_sec: i64,
//     tv_nsec: Nanoseconds,
// }

// struct Instant {
//     t: Timespec,
// }

struct Nanoseconds(u32);

struct Timespec {
    tv_sec: i64,
    tv_nsec: Nanoseconds,
}

pub(crate) fn to_std_instant(instant: u64) -> std::time::Instant {
    let dur = Duration::from_nanos(instant);

    unsafe {
        mem::transmute(Timespec {
            tv_sec: dur.as_secs() as i64,
            tv_nsec: Nanoseconds(dur.subsec_nanos()),
        })
    }
}

pub(crate) fn from_std_instant(instant: std::time::Instant) -> u64 {
    let ts: Timespec = unsafe { mem::transmute(instant) };

    ts.tv_sec as u64 * 1_000_000_000 + ts.tv_nsec.0 as u64
}