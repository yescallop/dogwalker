use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Seek, Write},
    path::PathBuf,
};

fn main() -> io::Result<()> {
    for entry in fs::read_dir("collect")? {
        let path = entry?.path();
        if path.is_file() {
            sort(path)?;
        }
    }
    Ok(())
}

fn sort(path: PathBuf) -> io::Result<()> {
    print!("{}: ", path.file_name().unwrap().to_str().unwrap());

    let file = File::options().write(true).read(true).open(path)?;
    let mut reader = BufReader::new(file);
    let mut vec = Vec::new();
    let mut num = 0;
    let mut last = 0;
    let mut sorted = true;

    loop {
        let buf = reader.fill_buf()?;
        if buf.is_empty() {
            break;
        }

        for &b in buf {
            if let Some(x) = (b as char).to_digit(10) {
                num = num * 10 + x;
            } else if b == b'\n' && num != 0 {
                vec.push(num);
                if num < last {
                    sorted = false;
                }
                last = num;
                num = 0;
            }
        }

        let len = buf.len();
        reader.consume(len);
    }

    if num != 0 {
        vec.push(num);
        if num < last {
            sorted = false;
        }
    }

    if sorted {
        println!("none");
    } else {
        vec.sort_unstable();

        let mut writer = BufWriter::new(reader.into_inner());
        writer.rewind()?;

        for num in vec {
            writeln!(writer, "{num}")?;
        }

        let pos = writer.stream_position()?;
        writer.into_inner()?.set_len(pos)?;
        println!("sorted");
    }

    Ok(())
}
