use crate::{set_recent, Clock};
use std::{
    fmt, io,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

static GLOBAL_UPKEEP_RUNNING: AtomicBool = AtomicBool::new(false);

/// Ultra-low-overhead access to slightly-delayed time.
///
/// In some applications, there can be a need to check the current time very often, so much so that
/// the overhead of checking the time can begin to eat up measurable overhead. For some of these
/// cases, the time may need to be accessed often but does not necessarily need to be incredibly
/// accurate: one millisecond granularity could be entirely acceptable.
///
/// For these cases, we provide a slightly-delayed version of the time to callers via
/// [`Clock::recent`], which is updated by a background upkeep thread.  That thread is configured
/// and spanwed via [`Upkeep`].
///
/// Given an update interval, [`Upkeep`] will faithfully attempt to update the global recent time
/// on the specified interval.  There is a trade-off to be struck in terms of how often the time is
/// updated versus the required accuracy.  Checking the time and updating the global reference is
/// itself not zero-cost, and so care must be taken to analyze the number of readers in order to
/// ensure that, given a particular update interval, the upkeep thread is saving more CPU time than
/// would be spent otherwise by directly querying the current time.
///
/// The recent time is read and written atomically.  It is global to an application, so if another
/// codepath creates the upkeep thread, the interval chosen by that instantiation will be the one
/// that all callers of [`Clock::recent`] end up using.
///
/// Multiple upkeep threads cannot exist at the same time.  A new upkeep thread can be started if
/// the old one is dropped and returns.
///
/// In terms of performance, reading the recent time can be up to two to three times as fast as
/// reading the current time in the optimized case of using the Time Stamp Counter source.  In
/// practice, while a caller might expect to take 12-14ns to read the TSC and scale it to reference
/// time, the recent time can be read in 4-5ns, with no reference scale conversion required.
#[derive(Debug)]
pub struct Upkeep {
    interval: Duration,
}

/// Handle to a running upkeep thread.
///
/// If a handle is dropped, the upkeep thread will be stopped, and the recent time will cease to
/// update.  The upkeep thread can be started again to resume updating the recent time.
#[derive(Debug)]
pub struct Handle {
    done: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

/// Errors thrown during the creation/spawning of the upkeep thread.
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// An upkeep thread is already running in this process.
    UpkeepRunning,
    /// An error occurred when trying to spawn the upkeep thread.
    FailedToSpawnUpkeepThread(io::Error),
    /// The upkeep thread could not be successfully pinned.
    FailedToPinUpkeepThread,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UpkeepRunning => write!(f, "upkeep thread already running"),
            Error::FailedToSpawnUpkeepThread(e) => {
                write!(f, "failed to spawn upkeep thread: {}", e)
            }
            Error::FailedToPinUpkeepThread => write!(f, "failed to pin upkeep thread"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UpkeepRunning => None,
            Self::FailedToSpawnUpkeepThread(e) => Some(e),
            Self::FailedToPinUpkeepThread => None,
        }
    }
}

impl Upkeep {
    /// Creates a new [`Upkeep`].
    pub fn new(interval: Duration) -> Upkeep {
        Upkeep { interval }
    }

    /// Creates a new [`Upkeep`] with the specified [`Clock`] instance.
    #[doc(hidden)]
    #[deprecated = "`Upkeep::new_with_clock` is not faster than `Upkeep::new`. Use `Upkeep::new` instead."]
    pub fn new_with_clock(interval: Duration, _: Clock) -> Upkeep {
        Upkeep { interval }
    }

    /// Start the upkeep thread, periodically updating the global coarse time.
    ///
    /// [`Handle`] represents a drop guard for the upkeep thread if it is successfully spawned.
    /// Dropping the handle will also instruct the upkeep thread to stop and exist, so the handle
    /// must be held while the upkeep thread should continue to run.
    ///
    /// # Errors
    ///
    /// If either an existing upkeep thread is running, or there was an issue when attempting to
    /// spawn the upkeep thread, an error variant will be returned describing the error.
    pub fn start(self) -> Result<Handle, Error> {
        self.inner_start(None)
    }

    /// Start the upkeep thread pinned to `core_id`, periodically updating the global coarse time.
    /// [`Upkeep`] will construct a [`Clock`] and run calibration against the core it is
    /// pinned to. Since all [`Clock`] instances share a global lazily initialized calibration,
    /// users intending to use this API should avoid calling [`Clock::new`] before starting a
    /// pinned [`Upkeep`] thread.
    ///
    /// [`Handle`] represents a drop guard for the upkeep thread if it is successfully spawned.
    /// Dropping the handle will also instruct the upkeep thread to stop and exist, so the handle
    /// must be held while the upkeep thread should continue to run.
    ///
    /// # Errors
    ///
    /// If either an existing upkeep thread is running, or there was an issue when attempting to
    /// spawn the upkeep thread, or the upkeep thread was not successfully pinned to `core_id`,
    /// an error variant will be returned describing the error.
    pub fn start_pinned(self, core_id: core_affinity::CoreId) -> Result<Handle, Error> {
        self.inner_start(Some(core_id))
    }

    fn inner_start(self, core_id: Option<core_affinity::CoreId>) -> Result<Handle, Error> {
        // If another upkeep thread is running, inform the caller.
        let _ = GLOBAL_UPKEEP_RUNNING
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| Error::UpkeepRunning)?;

        let interval = self.interval;

        let done = Arc::new(AtomicBool::new(false));
        let their_done = done.clone();

        let (pin_success_sender, pin_success_receiver) = mpsc::sync_channel(1);
        let result = thread::Builder::new()
            .name("quanta-upkeep".to_string())
            .spawn(move || {
                if let Some(core_id) = core_id {
                    let success = core_affinity::set_for_current(core_id);

                    // Panic safety: `send` may panic if the receiver side has been dropped.
                    // That can happen only if the parent thread paniced before we reached this
                    // point. So, this (practically) never panics.
                    pin_success_sender.send(success).unwrap();

                    // Do not keep this thread running if pinning was requested, but we failed to
                    // pin.
                    if !success {
                        GLOBAL_UPKEEP_RUNNING.store(false, Ordering::SeqCst);
                        return;
                    }
                }

                let clock = Clock::new();
                while !their_done.load(Ordering::Acquire) {
                    set_recent(clock.now());

                    thread::sleep(interval);
                }

                GLOBAL_UPKEEP_RUNNING.store(false, Ordering::SeqCst);
            })
            .map_err(Error::FailedToSpawnUpkeepThread);

        // Let another caller attempt to spawn the upkeep thread if we failed to do so.
        if result.is_err() {
            GLOBAL_UPKEEP_RUNNING.store(false, Ordering::SeqCst);
        }

        // When thread pinning is requested, verify `quanta-upkeep` has been successfully pinned.
        if core_id.is_some() {
            // Panic safety: `recv` may panic if the sender has disconnected, or is disconnecting
            // while this is blocking.
            // However, since we always send a message before the sender can be dropped, this call
            // never panics.
            let success = pin_success_receiver.recv().unwrap();
            if !success {
                return Err(Error::FailedToPinUpkeepThread);
            }
        }

        let handle = result?;

        Ok(Handle {
            done,
            handle: Some(handle),
        })
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.done.store(true, Ordering::Release);

        if let Some(handle) = self.handle.take() {
            let _result = handle
                .join()
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to stop upkeep thread"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Upkeep;
    use std::time::Duration;

    #[test]
    #[cfg_attr(target_arch = "wasm32", ignore)] // WASM is single threaded
    fn test_spawning_second_upkeep() {
        let first = Upkeep::new(Duration::from_millis(250)).start();
        let second = Upkeep::new(Duration::from_millis(250))
            .start()
            .map_err(|e| e.to_string());

        assert!(first.is_ok());

        let second_err = second.expect_err("second upkeep should be error, got handle");
        assert_eq!(second_err, "upkeep thread already running");
    }
}
