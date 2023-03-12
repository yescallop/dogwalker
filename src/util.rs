use std::{fmt, fs::File, io, path::Path};

#[cfg(windows)]
use std::os::windows::prelude::OpenOptionsExt;

use crate::Point;

pub fn size_of_steps(steps: &[Point<i64>]) -> u64 {
    steps
        .iter()
        .flat_map(|v| [v.x, v.y])
        .map(|n| n.unsigned_abs().pow(2))
        .sum()
}

pub fn create_file(path: impl AsRef<Path>) -> io::Result<File> {
    let mut options = File::options();
    #[cfg(windows)]
    options.share_mode(1);
    options.read(true).write(true).create(true).open(path)
}

pub struct Steps<'a>(pub &'a [Point<i64>]);

impl<'a> fmt::Display for Steps<'a> {
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
