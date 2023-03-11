use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    vec,
};

use once_cell::sync::OnceCell;
use regex::Regex;

use crate::Point;

#[derive(Debug)]
pub struct Record {
    pub si: u32,
    pub steps: Option<Vec<Point<i32>>>,
}

pub fn parse_record_file(path: impl AsRef<Path>) -> io::Result<Vec<Record>> {
    let mut reader = LineReader::new(BufReader::new(File::open(path)?));
    let mut recs = vec![];

    while let Some(line) = reader.read_line()? {
        let rec = parse_record_line(line)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid record"))?;
        recs.push(rec);
    }
    Ok(recs)
}

fn parse_record_line(line: &str) -> Option<Record> {
    static INT_RE: OnceCell<Regex> = OnceCell::new();
    let int_re = INT_RE.get_or_init(|| Regex::new("-?\\d+").unwrap());

    let (si, steps) = match line.split_once(':') {
        Some((si, steps)) => (si, Some(steps)),
        None => (line, None),
    };
    let si: u32 = si.parse().ok()?;
    let steps = steps.and_then(|steps| {
        let mut ints = int_re.find_iter(steps).map(|m| m.as_str()).peekable();
        let mut steps = vec![];

        while ints.peek().is_some() {
            steps.push(Point {
                x: ints.next()?.parse().ok()?,
                y: ints.next()?.parse().ok()?,
            });
        }
        Some(steps)
    });
    Some(Record { si, steps })
}

struct LineReader<R> {
    reader: R,
    buf: String,
}

impl<R: BufRead> LineReader<R> {
    fn new(reader: R) -> Self {
        Self {
            reader,
            buf: String::new(),
        }
    }

    fn read_line(&mut self) -> io::Result<Option<&str>> {
        self.buf.clear();
        if self.reader.read_line(&mut self.buf)? == 0 {
            return Ok(None);
        }

        if self.buf.ends_with('\n') {
            self.buf.pop();
            if self.buf.ends_with('\r') {
                self.buf.pop();
            }
        }
        Ok(Some(&self.buf[..]))
    }
}
