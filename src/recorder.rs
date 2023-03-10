use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    sync::{
        atomic::{AtomicBool, AtomicU64},
        RwLock,
    },
};

use mixhash::Mix;

use crate::{acc_size, Point};

struct RecorderState {
    si_map: HashMap<u32, u64, Mix>,
    file: File,
    buf: String,
}

#[readonly::make]
pub struct Recorder {
    pub n: usize,
    pub closed: bool,
    pub running: AtomicBool,
    pub count: AtomicU64,
    pub minify_more: bool,
    state: RwLock<RecorderState>,
}

impl Recorder {
    pub fn new(n: usize, closed: bool, minify_more: bool) -> io::Result<Recorder> {
        assert!(if closed { n > 2 } else { n > 0 });

        let kind = if closed { "closed" } else { "general" };
        let path = format!("record/{n}-{kind}.txt");
        let file = File::options()
            .write(true)
            .read(true)
            .create(true)
            .open(path)?;
        let mut reader = BufReader::new(file);
        let mut si_map = HashMap::with_hasher(Mix);
        let mut si_min = u32::MAX;

        let mut buf = String::new();

        while reader.read_line(&mut buf)? != 0 {
            if let Some((si, nums)) = buf.split_once(':') {
                let si = si.parse().unwrap();
                let size = nums
                    .matches(|ch: char| ch == '-' || ch.is_ascii_digit())
                    .filter_map(|n| n.parse::<i32>().ok())
                    .fold(0, acc_size);
                si_map.insert(si, size);
                si_min = si_min.min(si);
            }
            buf.clear();
        }

        println!("min: {si_min}");

        Ok(Recorder {
            n,
            closed,
            running: AtomicBool::new(true),
            count: AtomicU64::new(0),
            minify_more,
            state: RwLock::new(RecorderState {
                si_map,
                file: reader.into_inner(),
                buf: String::new(),
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
            buf,
        } = &mut *self.state.write().unwrap();
        si_set.insert(si, size);

        buf.clear();
        write_msg(si, steps, buf).unwrap();

        writeln!(file, "{}", buf).unwrap();
        println!("{}", buf);
    }
}

pub fn write_msg(cnt: u32, steps: &[Point<i32>], writer: &mut impl fmt::Write) -> fmt::Result {
    write!(writer, "{}: {{", cnt)?;
    for i in 0..steps.len() {
        if i != 0 {
            write!(writer, ",")?;
        }
        let step = steps[i];
        write!(writer, "{{{},{}}}", step.x, step.y)?;
    }
    write!(writer, "}}")
}
