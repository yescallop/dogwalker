use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
};

#[derive(Clone, Copy, Default)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

pub trait State: Sync + Send {
    fn update(&self, cnt: u32, steps: &[Point<i32>]);
}

pub fn print_new(cnt: u32, steps: &[Point<i32>]) {
    print!("{}: {{", cnt);
    for i in 0..steps.len() {
        if i != 0 {
            print!(",");
        }
        let step = steps[i];
        print!("{{{},{}}}", step.x, step.y);
    }
    println!("}}");
}

pub struct Min(AtomicU32);

impl Min {
    pub fn new() -> Min {
        Min(AtomicU32::new(u32::MAX))
    }
}

impl State for Min {
    fn update(&self, cnt: u32, steps: &[Point<i32>]) {
        let prev = self.0.fetch_min(cnt, Ordering::SeqCst);
        if cnt <= prev {
            print_new(cnt, steps);
        }
    }
}

pub struct Max(AtomicU32);

impl Max {
    pub fn new() -> Max {
        Max(AtomicU32::new(0))
    }
}

impl State for Max {
    fn update(&self, cnt: u32, steps: &[Point<i32>]) {
        let prev = self.0.fetch_max(cnt, Ordering::SeqCst);
        if cnt >= prev {
            print_new(cnt, steps);
        }
    }
}

pub struct Collect {
    file: Mutex<File>,
    set_min: Mutex<(HashSet<u32>, u32)>,
}

impl Collect {
    pub fn new(n: usize, close: bool) -> io::Result<Collect> {
        let kind = if close { "closed" } else { "general" };
        let path = format!("collect/{n}-{kind}.txt");
        let file = File::options()
            .write(true)
            .read(true)
            .create(true)
            .open(path)?;
        let mut reader = BufReader::new(file);
        let mut set = HashSet::new();
        let mut num = 0u32;
        let mut min = u32::MAX;

        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }

            for &b in buf {
                if let Some(x) = (b as char).to_digit(10) {
                    num = num * 10 + x;
                } else if b == b'\n' && num != 0 {
                    set.insert(num);
                    if num < min {
                        min = num;
                    }
                    num = 0;
                }
            }

            let len = buf.len();
            reader.consume(len);
        }

        let mut file = reader.into_inner();
        if num != 0 {
            set.insert(num);
            writeln!(file).unwrap();
        }

        println!("min: {min}");

        Ok(Collect {
            file: Mutex::new(file),
            set_min: Mutex::new((set, min)),
        })
    }
}

impl State for Collect {
    fn update(&self, cnt: u32, steps: &[Point<i32>]) {
        let mut set_min = self.set_min.lock().unwrap();
        if set_min.0.insert(cnt) {
            if cnt <= set_min.1 {
                set_min.1 = cnt;
                print_new(cnt, steps);
            } else {
                print!("{cnt},");
                io::stdout().flush().unwrap();
            }

            drop(set_min);
            let mut file = self.file.lock().unwrap();
            writeln!(file, "{cnt}").unwrap();
        }
    }
}
