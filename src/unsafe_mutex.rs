use std::cell::UnsafeCell;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed};
use std::thread::sleep;
use std::time::Duration;

/// A simple implementation of a Mutex based on spin lock
///
/// # Features
/// - The mutex requires manual unlocking
/// - It can be shared between threads
///
/// # Usage
/// ```
/// let mux = Mutex::new(520);
/// let data = mux.lock();
/// *data = 251;
/// mux.unlock();
/// ```
pub struct Mutex<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> &mut T {
        while let Err(_) = self.lock.compare_exchange(false, true, Acquire, Relaxed) {
            sleep(Duration::from_nanos(10));
        }
        let d = unsafe { &mut *self.data.get() };
        d
    }

    pub fn unlock(&self) {
        self.lock.store(false, Relaxed);
    }
}

unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}
