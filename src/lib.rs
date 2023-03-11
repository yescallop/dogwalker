mod parser;

mod recorder;
pub use recorder::Recorder;

mod rng;
use rng::Rng;

mod sort;
pub use sort::sort_records;

use std::{
    mem,
    sync::{atomic::Ordering, Arc},
};

use quickperm::meta::{Dyn, IndexPair, MetaPerm};

#[derive(Clone, Copy, Default, Debug)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

fn direction(i: Point<i64>, j: Point<i64>, k: Point<i64>) -> i64 {
    (k.x - i.x) * (j.y - i.y) - (j.x - i.x) * (k.y - i.y)
}

fn on_segment(i: Point<i64>, j: Point<i64>, k: Point<i64>) -> bool {
    i.x.min(j.x) <= k.x && k.x <= i.x.max(j.x) && i.y.min(j.y) <= k.y && k.y <= i.y.max(j.y)
}

#[allow(clippy::if_same_then_else, clippy::needless_bool)]
fn segments_intersect(p: [Point<i64>; 4]) -> bool {
    let d1 = direction(p[2], p[3], p[0]);
    let d2 = direction(p[2], p[3], p[1]);
    let d3 = direction(p[0], p[1], p[2]);
    let d4 = direction(p[0], p[1], p[3]);

    if ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0)) && ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0)) {
        true
    } else if d1 == 0 && on_segment(p[2], p[3], p[0]) {
        true
    } else if d2 == 0 && on_segment(p[2], p[3], p[1]) {
        true
    } else if d3 == 0 && on_segment(p[0], p[1], p[2]) {
        true
    } else if d4 == 0 && on_segment(p[0], p[1], p[3]) {
        true
    } else {
        false
    }
}

pub struct Walker {
    mp: MetaPerm<Dyn>,
    steps: Vec<Point<i32>>,
    walk: Vec<Point<i64>>,
    steps_buf: Vec<Point<i32>>,
    closed: bool,
}

impl Default for Walker {
    fn default() -> Self {
        Self::new(3, false)
    }
}

impl Walker {
    pub fn new(n: usize, closed: bool) -> Self {
        assert!(n > 2);
        Self {
            mp: MetaPerm::new(n),
            steps: vec![Point::default(); n],
            walk: vec![Point::default(); n + 1],
            steps_buf: vec![],
            closed,
        }
    }

    pub fn set_steps(&mut self, steps: &[Point<i32>]) {
        let n = steps.len();
        assert!(n > 2);

        if self.mp.len() != n {
            self.mp = MetaPerm::new(n);
        }

        self.steps.clear();
        self.steps.extend_from_slice(steps);
        self.walk.resize_with(n + 1, Point::default);
        self.closed = self.is_walk_closed();
    }

    pub fn steps(&self) -> &[Point<i32>] {
        &self.steps
    }

    fn is_walk_simple(&mut self) -> bool {
        let mut v = Point::default();
        let n = self.steps.len();
        for i in 0..n {
            let step = self.steps[i];
            let last_v = v;

            v.x += step.x as i64;
            v.y += step.y as i64;
            self.walk[i + 1] = v;

            if i >= 2 {
                let start = (i == n - 1 && v.x == 0 && v.y == 0) as usize;
                for j in start..i - 1 {
                    if segments_intersect([last_v, v, self.walk[j], self.walk[j + 1]]) {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn simpleness_index(&mut self) -> u32 {
        let mut si = 0;
        loop {
            si += self.is_walk_simple() as u32;
            unsafe { IndexPair::new(0, 1).swap_unchecked(&mut self.steps) }
            si += self.is_walk_simple() as u32;
            if let Some(p) = self.mp.gen_even() {
                unsafe { p.swap_unchecked(&mut self.steps) }
            } else {
                break;
            }
        }
        si
    }

    pub fn is_walk_closed(&self) -> bool {
        self.steps.iter().fold((0, 0), |(x, y), step| {
            (x + step.x as i64, y + step.y as i64)
        }) == (0, 0)
    }

    pub fn has_collinear_steps(&self) -> bool {
        let n = self.steps.len();
        for i in 0..n {
            let a = self.steps[i];
            for j in i + 1..n {
                let b = self.steps[j];
                if (a.x as i64) * (b.y as i64) == (a.y as i64) * (b.x as i64) {
                    return true;
                }
            }
        }
        false
    }

    pub fn minify_steps(&mut self, si: u32) -> bool {
        let mut minified = false;
        self.steps_buf.clone_from(&self.steps);

        loop {
            let mut v = Point::<i32>::default();
            self.steps
                .iter_mut()
                .zip(&mut self.steps_buf)
                .for_each(|(step, step_buf)| {
                    step.x = step_buf.x / 2;
                    step.y = step_buf.y / 2;
                    v.x += step.x;
                    v.y += step.y;
                });
            if self.closed {
                let last = self.steps.last_mut().unwrap();
                last.x -= v.x;
                last.y -= v.y;
            }

            let good = !self.has_collinear_steps() && self.simpleness_index() == si;
            mem::swap(&mut self.steps_buf, &mut self.steps);
            if !good {
                break;
            }
            minified = true;
        }

        if self.steps.iter().map(|v| v.x.signum()).sum::<i32>() < 0 {
            self.steps.iter_mut().for_each(|v| v.x = -v.x);
            minified = true;
        }

        if self.steps.iter().map(|v| v.y.signum()).sum::<i32>() < 0 {
            self.steps.iter_mut().for_each(|v| v.y = -v.y);
            minified = true;
        }

        minified
    }
}

pub struct Simulator {
    rng: Rng,
    walker: Walker,
    recorder: Arc<Recorder>,
}

const SHIFTS: u32 = 16;

impl Simulator {
    pub fn new(rec: Arc<Recorder>) -> Self {
        Self {
            rng: Rng::new(),
            walker: Walker::new(rec.n, rec.closed),
            recorder: rec,
        }
    }

    fn gen(&mut self) {
        if !self.recorder.closed {
            for step in &mut self.walker.steps {
                let x = self.rng.gen() as i32 >> SHIFTS;
                let y = self.rng.gen() as i32 >> SHIFTS;
                *step = Point { x, y };
            }
        } else {
            let mut v = Point::<i32>::default();
            let (last, steps) = self.walker.steps.split_last_mut().unwrap();

            for step in steps {
                let x = self.rng.gen() as i32 >> SHIFTS;
                let y = self.rng.gen() as i32 >> SHIFTS;
                *step = Point { x, y };
                v.x += x;
                v.y += y;
            }

            (v.x, v.y) = (-v.x, -v.y);
            *last = v;
        }
    }

    pub fn run(&mut self) {
        loop {
            self.gen();
            let si = self.walker.simpleness_index();
            if !self.recorder.minify {
                if !self.recorder.contains(si) && !self.walker.has_collinear_steps() {
                    self.walker.minify_steps(si);
                    self.recorder.insert(si, &self.walker.steps, 0);
                }
            } else {
                self.walker.minify_steps(si);
                let size = size_of_steps(&self.walker.steps);
                if !self.recorder.contains_smaller(si, size) && !self.walker.has_collinear_steps() {
                    self.recorder.insert(si, &self.walker.steps, size);
                }
            }

            if !self.recorder.running.load(Ordering::SeqCst) {
                return;
            }
            self.recorder.count.fetch_add(1, Ordering::SeqCst);
        }
    }
}

fn size_of_steps(steps: &[Point<i32>]) -> u64 {
    steps
        .iter()
        .flat_map(|v| [v.x, v.y])
        .map(|n| (n.unsigned_abs() as u64).pow(2))
        .sum()
}
