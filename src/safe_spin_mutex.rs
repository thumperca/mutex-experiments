use std::cell::UnsafeCell;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed};
use std::thread::sleep;
use std::time::Duration;

/// MutexGuard to automate unlocking of held lock
/// The lock is unlocked when MutexGuard is dropped
pub struct MutexGuard<'a, T> {
    inner: &'a Mutex<T>,
}

impl<'a, T> MutexGuard<'a, T> {
    pub fn new(mux: &'a Mutex<T>) -> Self {
        Self { inner: mux }
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.inner.data.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.data.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.inner.lock.store(false, Relaxed)
    }
}

impl<'a, T> Display for MutexGuard<'a, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let d = unsafe { self.inner.data.get().read() };
        write!(f, "{}", d)
    }
}

/// A safe implementation of a Mutex based on spin lock
///
/// # Features
/// - Unlocking is automated with MutexGuard
/// - It can be shared between threads
///
/// # Usage
/// ```
/// let mux = Mutex::new(520);
/// let mut data = mux.lock();
/// *data = 251;
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

    pub fn lock(&self) -> MutexGuard<T> {
        while let Err(_) = self.lock.compare_exchange(false, true, Acquire, Relaxed) {
            sleep(Duration::from_nanos(10));
        }
        MutexGuard::new(self)
    }
}

unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

mod tests {
    use super::*;

    fn it_works() {
        let mux = Mutex::new(100);
        let mut d = mux.lock();
        *d += 1;
        drop(d);
        let d = mux.lock();
        assert_eq!(d, 101);
    }
}
