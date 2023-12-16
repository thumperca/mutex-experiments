use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

pub struct Mutex<'a, T> {
    lock: AtomicBool,
    data: &'a mut T,
}

impl<'a, T> Mutex<'a, T> {
    pub fn new(value: T) -> Self {
        let value = Box::leak(Box::new(value));
        Self {
            lock: AtomicBool::new(false),
            data: value,
        }
    }

    // pub fn lock(&self) -> &mut T {
    //     while let Err(_) = self.lock.compare_exchange(false, true, Acquire, Relaxed) {
    //         sleep(Duration::from_nanos(10));
    //     }
    //     let d = unsafe { &mut *self.data };
    //     d
    // }

    pub fn unlock(&self) {
        self.lock.store(false, Relaxed);
    }
}

impl<'a, T> Drop for Mutex<'a, T> {
    fn drop(&mut self) {
        let value = unsafe { Box::from_raw(self.data) };
        drop(value);
    }
}
