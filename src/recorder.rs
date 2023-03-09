use std::{
    collections::HashSet,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    sync::RwLock,
};

use crate::Point;
use mixhash::Mix;

struct RecorderState {
    si_set: HashSet<u32, Mix>,
    file: File,
    buf: String,
}

#[readonly::make]
pub struct Recorder {
    pub n: usize,
    pub closed: bool,
    state: RwLock<RecorderState>,
}

impl Recorder {
    pub fn new(n: usize, closed: bool) -> io::Result<Recorder> {
        assert!(if closed { n > 2 } else { n > 0 });

        let kind = if closed { "closed" } else { "general" };
        let path = format!("record/{n}-{kind}.txt");
        let file = File::options()
            .write(true)
            .read(true)
            .create(true)
            .open(path)?;
        let mut reader = BufReader::new(file);
        let mut si_set = HashSet::with_hasher(Mix);
        let mut si_min = u32::MAX;

        let mut buf = String::new();

        while reader.read_line(&mut buf)? != 0 {
            if let Some(si) = buf
                .split_once(':')
                .and_then(|(si, _)| si.parse::<u32>().ok())
            {
                si_set.insert(si);
                si_min = si_min.min(si);
            }
            buf.clear();
        }

        println!("min: {si_min}");

        Ok(Recorder {
            n,
            closed,
            state: RwLock::new(RecorderState {
                si_set,
                file: reader.into_inner(),
                buf: String::new(),
            }),
        })
    }

    pub fn contains(&self, si: u32) -> bool {
        self.state.read().unwrap().si_set.contains(&si)
    }

    pub fn insert(&self, si: u32, steps: &[Point<i32>]) {
        let RecorderState { si_set, file, buf } = &mut *self.state.write().unwrap();
        if !si_set.insert(si) {
            return;
        }

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
