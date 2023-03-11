use std::{
    collections::{btree_map::Entry, BTreeMap},
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::PathBuf,
};

use crate::{
    parser::{parse_record_file, Record},
    size_of_steps,
    util::Steps,
    Walker,
};

pub fn sort_records() -> io::Result<()> {
    let mut walker = Walker::default();
    for entry in fs::read_dir("record")? {
        sort(entry?.path(), &mut walker)?;
    }
    Ok(())
}

fn sort(path: PathBuf, walker: &mut Walker) -> io::Result<()> {
    let name = path.file_stem().unwrap().to_str().unwrap();
    let closed = name.ends_with("closed");
    print!("{name}: ");

    let mut verify_and_minify = |si, steps: &mut _| {
        walker.set_steps(steps);

        assert!(!walker.has_collinear_steps());
        assert!(!closed || walker.is_walk_closed());
        assert_eq!(si, walker.simpleness_index());

        if walker.minify_steps(si) {
            steps.copy_from_slice(walker.steps());
            true
        } else {
            false
        }
    };

    let records = parse_record_file(&mut File::open(&path)?)?;

    let mut map = BTreeMap::new();
    let mut last_si = 0;
    let mut sorted = true;

    for Record { si, mut steps } in records {
        match map.entry(si) {
            Entry::Vacant(e) => {
                if let Some(steps) = &mut steps {
                    if verify_and_minify(si, steps) {
                        sorted = false;
                    }
                }
                e.insert(steps);
            }
            Entry::Occupied(mut e) => {
                match (e.get(), steps) {
                    (None, Some(mut steps)) => {
                        verify_and_minify(si, &mut steps);
                        e.insert(Some(steps));
                    }
                    (Some(e_steps), Some(mut steps)) => {
                        verify_and_minify(si, &mut steps);
                        if size_of_steps(&steps) < size_of_steps(e_steps) {
                            e.insert(Some(steps));
                        }
                    }
                    _ => (),
                }
                sorted = false;
            }
        }
        if si <= last_si {
            sorted = false;
        }
        last_si = si;
    }

    if sorted {
        println!("unchanged");
    } else {
        let mut bw = BufWriter::new(File::create(path)?);
        for (si, steps) in map {
            if let Some(steps) = steps {
                writeln!(bw, "{si}: {}", Steps(&steps))?;
            } else {
                writeln!(bw, "{si}")?;
            }
        }
        println!("sorted");
    }

    Ok(())
}
