#![allow(dead_code)]
use crate::ClockSource;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

/// Type which can be converted into a nanosecond representation.
///
/// This allows users of [`Mock`] to increment/decrement the time both with raw integer values and
/// the more convenient [`Duration`] type.
pub trait IntoNanoseconds {
    fn into_nanos(self) -> u64;
}

impl IntoNanoseconds for u64 {
    fn into_nanos(self) -> u64 {
        self
    }
}

impl IntoNanoseconds for Duration {
    fn into_nanos(self) -> u64 {
        self.as_nanos() as u64
    }
}

/// Controllable time source for use in tests.
#[derive(Debug, Clone)]
pub struct Mock {
    offset: Arc<AtomicU64>,
}

impl Mock {
    pub(crate) fn new() -> Self {
        Self {
            offset: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Increments the time by the given amount.
    pub fn increment<N: IntoNanoseconds>(&self, amount: N) {
        self.offset.fetch_add(amount.into_nanos(), Ordering::Release);
    }

    /// Decrements the time by the given amount.
    pub fn decrement<N: IntoNanoseconds>(&self, amount: N) {
        self.offset.fetch_sub(amount.into_nanos(), Ordering::Release);
    }
}

impl ClockSource for Mock {
    fn now(&self) -> u64 {
        self.offset.load(Ordering::Acquire)
    }

    fn start(&self) -> u64 {
        self.now()
    }

    fn end(&self) -> u64 {
        self.now()
    }
}
