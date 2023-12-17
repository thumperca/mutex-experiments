use crate::raw_mutex::Mutex as RawMutex;
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread::Thread;

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
        let queue = self.inner.queue.lock();
        let thread = queue.pop_front();
        // SAFETY: no writing to queue after unlock
        unsafe {
            self.inner.queue.unlock();
        }
        self.inner.lock.store(false, Release);
        if let Some(thread) = thread {
            thread.unpark();
        }
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

/// A safe implementation of a Mutex without using a spin lock
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
    queue: RawMutex<VecDeque<Thread>>,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(value),
            queue: RawMutex::new(VecDeque::new()),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        while let Err(_) = self.lock.compare_exchange(false, true, Acquire, Relaxed) {
            let thread = std::thread::current();
            // add thread to queue
            let queue = self.queue.lock();
            let mut exists = false;
            for item in queue.iter() {
                if item.id() == thread.id() {
                    exists = true;
                    break;
                }
            }
            if !exists {
                queue.push_back(thread);
            }
            // SAFETY: thread is going to sleep
            // so, it will not be writing to queue again without re-acquiring the lock
            unsafe {
                self.queue.unlock();
            }
            std::thread::park();
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

    #[test]
    fn array() {
        let mux = Arc::new(Mutex::new(Vec::with_capacity(1000)));
        // spawn 10,000 thread, each trying to increment the data held in the mutex
        std::thread::scope(|scope| {
            for i in 0..1000 {
                let mute = mux.clone();
                scope.spawn(move || {
                    let mut lock = mute.lock();
                    lock.push(i);
                });
            }
        });
        // test final number
        let d = mux.lock();
        assert_eq!(d.len(), 1000);
    }
}
