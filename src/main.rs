mod raw_mutex;
mod unsafe_mutex;

use std::sync::Arc;
use std::thread;
use unsafe_mutex::Mutex;

fn main() {
    let m = Arc::new(Mutex::new(520));

    thread::scope(|scope| {
        let mx = m.clone();
        let mx2 = m.clone();

        scope.spawn(move || {
            let data = mx.lock();
            *data = 420;
            mx.unlock();
        });

        scope.spawn(move || {
            let data = mx2.lock();
            *data = 840;
            mx2.unlock();
        });
    });

    let data = m.lock();
    println!("Hello, world! {data}");
}
