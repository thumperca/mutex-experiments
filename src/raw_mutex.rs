use std::hint::spin_loop;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed};

/// An unsafe implementation of Mutex using Raw Pointers
///
/// ## Features
/// - Simple
/// - Requires manual unlocking
///
/// ## Usage
/// ```
/// let mux = Mutex::new(100);
/// let mut d = mux.lock();
/// *d += 1
/// mux.unlock();
/// ```
pub struct Mutex<T> {
    lock: AtomicBool,
    data: *mut T,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        let value = Box::into_raw(Box::new(value));
        Self {
            lock: AtomicBool::new(false),
            data: value,
        }
    }

    pub fn lock(&self) -> &mut T {
        while let Err(_) = self.lock.compare_exchange(false, true, Acquire, Relaxed) {
            spin_loop();
        }
        let d = unsafe { &mut *self.data };
        d
    }

    pub unsafe fn unlock(&self) {
        self.lock.store(false, Relaxed);
    }
}

impl<T> Drop for Mutex<T> {
    fn drop(&mut self) {
        let value = unsafe { Box::from_raw(self.data) };
        drop(value);
    }
}

unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn it_works() {
        let mux = Arc::new(Mutex::new(100));
        // update mutex
        std::thread::scope(|scope| {
            let m1 = mux.clone();
            let m2 = mux.clone();
            scope.spawn(move || {
                let d = m1.lock();
                *d += 1;
                unsafe {
                    m1.unlock();
                }
            });
            scope.spawn(move || {
                let d = m2.lock();
                *d += 1;
                unsafe {
                    m2.unlock();
                }
            });
        });
        // check update
        let d = mux.lock();
        assert_eq!(102, *d);
    }
}
