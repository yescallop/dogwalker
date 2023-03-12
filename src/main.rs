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

    let core_ids = core_affinity::get_core_ids().unwrap();

    let jobs = args
        .jobs
        .map(|j| j.get())
        .filter(|&j| j <= core_ids.len())
        .unwrap_or(core_ids.len());
    println!("jobs: {jobs}");

    let recorder = Arc::new(Recorder::new(
        args.n.unwrap() as usize,
        args.closed,
        args.minify,
    )?);
    let start = Instant::now();

    let handles = core_ids
        .into_iter()
        .take(jobs)
        .map(|id| {
            let rec = recorder.clone();
            thread::spawn(move || {
                if core_affinity::set_for_current(id) {
                    Simulator::new(rec).run();
                }
            })
        })
        .collect::<Vec<_>>();

    ctrlc::set_handler(move || {
        recorder.running.store(false, Ordering::Relaxed);
        let count = recorder.count.load(Ordering::Relaxed);
        let speed = count as f64 / start.elapsed().as_secs_f64();

        println!("count: {count} ({speed:.0}/s)");
    })
    .expect("Error setting Ctrl-C handler");

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
