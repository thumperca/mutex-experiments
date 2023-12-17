mod mutex;
mod raw_mutex;
mod safe_spin_mutex;
mod unsafe_spin_mutex;

use mutex::Mutex;
use std::sync::Arc;
use std::thread;

fn main() {
    let m = Arc::new(Mutex::new(520));

    thread::scope(|scope| {
        let mx = m.clone();
        let mx2 = m.clone();

        scope.spawn(move || {
            let mut data = mx.lock();
            *data = 420;
        });

        scope.spawn(move || {
            let mut data = mx2.lock();
            *data = 840;
        });
    });

    let data = m.lock();
    println!("Hello, world! {}", data);
}
