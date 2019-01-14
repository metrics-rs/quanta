#![allow(dead_code)]
use crate::ClockSource;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

/// Controllable time source for use in tests.
#[derive(Clone)]
pub struct Mock {
    offset: Arc<AtomicUsize>,
}

impl Mock {
    pub(crate) fn new(offset: usize) -> Self {
        Self {
            offset: Arc::new(AtomicUsize::new(offset)),
        }
    }

    /// Increments the time by the given amount.
    pub fn increment(&self, amount: usize) { self.offset.fetch_add(amount, Ordering::Release); }

    /// Decrements the time by the given amount.
    pub fn decrement(&self, amount: usize) { self.offset.fetch_sub(amount, Ordering::Release); }
}

impl ClockSource for Mock {
    fn now(&self) -> u64 { self.offset.load(Ordering::Acquire) as u64 }

    fn start(&self) -> u64 { self.now() }

    fn end(&self) -> u64 { self.now() }
}
