use std::{
    io,
    num::NonZeroUsize,
    sync::{atomic::Ordering, Arc},
    thread,
    time::Instant,
};

use clap::Parser;
use dogwalker::{Recorder, Simulator};

#[derive(Parser)]
struct Args {
    /// Search for closed walks
    #[arg(short, long)]
    closed: bool,
    /// Search while minifying the existing records
    #[arg(short, long)]
    minify: bool,
    /// Sort all the records and exit
    #[arg(short, long, conflicts_with_all(["closed", "minify", "jobs", "n"]))]
    sort: bool,
    /// Number of parallel jobs
    #[arg(short)]
    jobs: Option<NonZeroUsize>,
    /// Number of steps to simulate
    #[arg(value_parser = clap::value_parser!(u8).range(3..), required_unless_present("sort"))]
    n: Option<u8>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    if args.sort {
        return dogwalker::sort_records();
    }

    let jobs = args.jobs.map(|j| j.get()).unwrap_or_else(num_cpus::get);
    println!("jobs: {jobs}");

    let recorder = Arc::new(Recorder::new(
        args.n.unwrap() as usize,
        args.closed,
        args.minify,
    )?);
    let mut handles = vec![];
    let start = Instant::now();

    for _ in 0..jobs {
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
