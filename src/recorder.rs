use std::{
    collections::HashMap,
    fmt::{self, Display},
    fs::File,
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, AtomicU64},
        RwLock,
    },
};

use mixhash::Mix;

use crate::{parser::parse_record_file, size_of_steps, Point};

struct RecorderState {
    si_map: HashMap<u32, u64, Mix>,
    file: File,
}

#[readonly::make]
pub struct Recorder {
    pub n: usize,
    pub closed: bool,
    pub running: AtomicBool,
    pub count: AtomicU64,
    pub minify: bool,
    state: RwLock<RecorderState>,
}

impl Recorder {
    pub fn new(n: usize, closed: bool, minify: bool) -> io::Result<Recorder> {
        assert!(n > 2);

        let kind = if closed { "closed" } else { "general" };
        let path = format!("record/{n}-{kind}.txt");

        let records = parse_record_file(&path)?;

        let mut si_min = u32::MAX;

        let si_map = records
            .iter()
            .filter_map(|rec| Some((rec.si, size_of_steps(rec.steps.as_ref()?))))
            .inspect(|&(si, _)| si_min = si_min.min(si))
            .collect();

        println!("min: {si_min}");

        Ok(Recorder {
            n,
            closed,
            running: AtomicBool::new(true),
            count: AtomicU64::new(0),
            minify,
            state: RwLock::new(RecorderState {
                si_map,
                file: File::options().append(true).open(&path)?,
            }),
        })
    }

    pub fn contains(&self, si: u32) -> bool {
        self.state.read().unwrap().si_map.contains_key(&si)
    }

    pub fn contains_smaller(&self, si: u32, size: u64) -> bool {
        if let Some(&x) = self.state.read().unwrap().si_map.get(&si) {
            x <= size
        } else {
            false
        }
    }

    pub fn insert(&self, si: u32, steps: &[Point<i32>], size: u64) {
        let RecorderState {
            si_map: si_set,
            file,
        } = &mut *self.state.write().unwrap();
        si_set.insert(si, size);

        writeln!(file, "{si}: {}", Steps(steps)).unwrap();
        println!("{si}: {}", Steps(steps));
    }
}

pub struct Steps<'a>(pub &'a [Point<i32>]);

impl<'a> Display for Steps<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (i, step) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ",")?;
            }
            write!(f, "{{{},{}}}", step.x, step.y)?;
        }
        write!(f, "}}")
    }
}
