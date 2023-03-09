use std::{
    sync::{atomic::Ordering, Arc},
    thread,
    time::Instant,
};

use dogwalker::{Recorder, Simulator};

fn main() {
    let recorder = Arc::new(Recorder::new(7, true).unwrap());
    let mut handles = vec![];
    let start = Instant::now();

    for _ in 0..4 {
        let recorder = recorder.clone();
        handles.push(thread::spawn(|| Simulator::new(recorder).run()));
    }

    ctrlc::set_handler(move || {
        recorder.running.store(false, Ordering::SeqCst);
        let count = recorder.count.load(Ordering::SeqCst);
        let speed = count / start.elapsed().as_secs();

        println!("count: {count} ({speed}/s)");
    })
    .expect("Error setting Ctrl-C handler");

    for handle in handles {
        handle.join().unwrap();
    }
}
