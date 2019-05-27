use crate::Clock;
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

/// Builder for creating an upkeep task.
#[derive(Debug)]
pub struct Builder {
    interval: Duration,
    clock: Clock,
}

/// Handle to a running upkeep task.
#[derive(Debug)]
pub struct Handle {
    done: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Builder {
    /// Creates a new [`Builder'], with a dedicated [`Clock`] instance.
    pub fn new(interval: Duration) -> Builder {
        Self::new_with_clock(interval, Clock::new())
    }

    /// Creates a new [`Builder`] with the specified [`Clock`] instance.
    pub fn new_with_clock(interval: Duration, clock: Clock) -> Builder {
        Builder { interval, clock }
    }

    /// Start the upkeep thread, periodically updating the global coarse time.
    ///
    /// If the return value is [`Ok(handle)`], then the thread was spawned successfully and can be
    /// stopped by dropping the returned handle.  Otherwise, [`Err`] contains the error that was
    /// returned when trying to spawn the thread.
    pub fn start(self) -> Result<Handle, io::Error> {
        let interval = self.interval;
        let clock = self.clock;

        let done = Arc::new(AtomicBool::new(false));
        let their_done = done.clone();

        let handle = thread::Builder::new()
            .name("quanta-upkeep".to_string())
            .spawn(move || {
                while !their_done.load(Ordering::Acquire) {
                    let now = clock.now();
                    Clock::upkeep(now);

                    thread::sleep(interval);
                }
            })?;

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
            let _ = handle
                .join()
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed to stop upkeep thread"));
        }
    }
}
