use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::PathBuf,
};

pub fn main() -> io::Result<()> {
    for entry in fs::read_dir("record")? {
        let path = entry?.path();
        if path.is_file() {
            sort(path)?;
        }
    }
    Ok(())
}

fn sort(path: PathBuf) -> io::Result<()> {
    print!("{}: ", path.file_name().unwrap().to_str().unwrap());

    let string = fs::read_to_string(&path)?;
    let mut map = BTreeMap::new();
    let mut last_si = 0;
    let mut sorted = true;

    for line in string.lines() {
        let si: u32 = line
            .split_once(':')
            .map(|(si, _)| si)
            .unwrap_or(line)
            .parse()
            .unwrap();
        map.insert(si, line);
        if si <= last_si {
            sorted = false;
        }
        last_si = si;
    }

    if sorted {
        println!("none");
    } else {
        let mut writer = BufWriter::new(File::create(&path)?);
        for (_, line) in map {
            writeln!(writer, "{line}")?;
        }
        println!("sorted");
    }

    Ok(())
}
