use std::{sync::Arc, thread};

use dogwalker::{Recorder, Simulator};

fn main() {
    let recorder = Arc::new(Recorder::new(6, false).unwrap());
    for _ in 0..8 {
        let recorder = recorder.clone();
        thread::spawn(|| Simulator::new(recorder).run());
    }
    thread::park();
}
