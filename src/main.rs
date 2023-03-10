use std::{
    env, io,
    sync::{atomic::Ordering, Arc},
    thread,
    time::Instant,
};

use dogwalker::{sort, Recorder, Simulator};

fn main() -> io::Result<()> {
    let arg = env::args().nth(1).unwrap();
    let mut arg = &*arg;

    if arg == "s" {
        return sort::main();
    }
    let mut minify_more = false;
    if let Some(rest) = arg.strip_prefix('m') {
        minify_more = true;
        arg = rest;
    }
    let mut closed = false;
    if let Some(rest) = arg.strip_suffix('c') {
        closed = true;
        arg = rest;
    }
    let n = arg.parse().unwrap();

    let recorder = Arc::new(Recorder::new(n, closed, minify_more)?);
    let mut handles = vec![];
    let start = Instant::now();

    for _ in 0..4 {
        let recorder = recorder.clone();
        handles.push(thread::spawn(|| Simulator::new(recorder).run()));
    }

    ctrlc::set_handler(move || {
        recorder.running.store(false, Ordering::SeqCst);
        let count = recorder.count.load(Ordering::SeqCst);
        let speed = count as f64 / start.elapsed().as_secs_f64();

        println!("count: {count} ({speed:.0}/s)");
    })
    .expect("Error setting Ctrl-C handler");

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
