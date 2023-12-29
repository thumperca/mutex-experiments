mod fair_mutex;
mod raw_mutex;
mod safe_spin_mutex;
mod unsafe_spin_mutex;

use fair_mutex::Mutex;
use std::sync::Arc;
use std::thread;

fn main() {
    println!("Started..");
    let m = Arc::new(Mutex::new(1));

    thread::scope(|scope| {
        for i in 0..100 {
            let mux = m.clone();
            scope.spawn(move || {
                let mut d = mux.lock();
                *d += 1;
                // sleep while holding lock
                std::thread::sleep(std::time::Duration::from_millis(20));
            });
        }
    });

    let data = m.lock();
    debug_assert!(*data == 101);
    println!("finished with {}", *data);
}
