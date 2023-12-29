use atomic_wait::{wait, wake_one};
use std::cell::UnsafeCell;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

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
        self.inner.lock.store(0, Release);
        wake_one(&self.inner.lock)
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

pub struct Mutex<T> {
    lock: AtomicU32,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            lock: AtomicU32::new(0),
            data: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        while let Err(_) = self.lock.compare_exchange(0, 1, Acquire, Relaxed) {
            wait(&self.lock, 1);
        }
        MutexGuard::new(self)
    }
}

unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn it_works() {
        let mux = Arc::new(Mutex::new(101));
        // spawn 10,000 thread, each trying to increment the data held in the mutex
        std::thread::scope(|scope| {
            for i in 0..10_000 {
                let mute = mux.clone();
                scope.spawn(move || {
                    let mut lock = mute.lock();
                    *lock += 1;
                });
            }
        });
        // test final number
        let d = mux.lock();
        assert_eq!(*d, 10_101);
    }
}
