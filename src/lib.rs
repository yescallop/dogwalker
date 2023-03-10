pub mod recorder;
pub mod rng;
pub mod sort;

#[derive(Clone, Copy, Default)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

use std::{
    mem,
    sync::{atomic::Ordering, Arc},
};

use quickperm::meta::{Dyn, IndexPair, MetaPerm};
pub use recorder::Recorder;
use rng::Rng;

fn direction(i: Point<i64>, j: Point<i64>, k: Point<i64>) -> i64 {
    (k.x - i.x) * (j.y - i.y) - (j.x - i.x) * (k.y - i.y)
}

fn on_segment(i: Point<i64>, j: Point<i64>, k: Point<i64>) -> bool {
    i.x.min(j.x) <= k.x && k.x <= i.x.max(j.x) && i.y.min(j.y) <= k.y && k.y <= i.y.max(j.y)
}

fn segments_intersect<const STRICT: bool>(p: [Point<i64>; 4]) -> bool {
    let d1 = direction(p[2], p[3], p[0]);
    let d2 = direction(p[2], p[3], p[1]);
    let d3 = direction(p[0], p[1], p[2]);
    let d4 = direction(p[0], p[1], p[3]);

    if !STRICT {
        (d1 ^ d2) & (d3 ^ d4) < 0
    } else if ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0))
        && ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0))
    {
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

pub struct Simulator {
    n: usize,
    closed: bool,
    rng: Rng,
    mp: MetaPerm<Dyn>,
    steps: Vec<Point<i32>>,
    walk: Vec<Point<i64>>,
    recorder: Arc<Recorder>,
    steps_buf: Vec<Point<i32>>,
}

const SHIFTS: u32 = 16;

impl Simulator {
    pub fn new(recorder: Arc<Recorder>) -> Self {
        let n = recorder.n;
        Self {
            n,
            closed: recorder.closed,
            rng: Rng::new(),
            mp: MetaPerm::new(n),
            steps: vec![Point::default(); n],
            walk: vec![Point::default(); n + 1],
            recorder,
            steps_buf: vec![],
        }
    }

    fn gen(&mut self) {
        if !self.closed {
            for i in 0..self.n {
                let x = self.rng.gen() as i32 >> SHIFTS;
                let y = self.rng.gen() as i32 >> SHIFTS;
                self.steps[i] = Point { x, y };
            }
        } else {
            let mut v = Point::<i32>::default();
            for i in 0..self.n - 1 {
                let x = self.rng.gen() as i32 >> SHIFTS;
                let y = self.rng.gen() as i32 >> SHIFTS;
                self.steps[i] = Point { x, y };
                v.x += x;
                v.y += y;
            }
            (v.x, v.y) = (-v.x, -v.y);
            self.steps[self.n - 1] = v;
        }
    }

    fn is_walk_simple<const STRICT: bool>(&mut self) -> bool {
        let mut v = Point::default();
        for i in 0..self.n {
            let step = self.steps[i];
            let last_v = v;

            v.x += step.x as i64;
            v.y += step.y as i64;
            self.walk[i + 1] = v;

            if i >= 2 {
                let start = (self.closed && i == self.n - 1) as usize;
                for j in start..i - 1 {
                    if segments_intersect::<STRICT>([last_v, v, self.walk[j], self.walk[j + 1]]) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn simpleness_index<const STRICT: bool>(&mut self) -> u32 {
        let mut si = 0;
        loop {
            si += self.is_walk_simple::<STRICT>() as u32;
            unsafe { IndexPair::new(0, 1).swap_unchecked(&mut self.steps) }
            si += self.is_walk_simple::<STRICT>() as u32;
            if let Some(p) = self.mp.gen_even() {
                unsafe { p.swap_unchecked(&mut self.steps) }
            } else {
                break;
            }
        }
        si
    }

    fn steps_noncollinear(&self) -> bool {
        for i in 0..self.n {
            for j in i + 1..self.n {
                let a = self.steps[i];
                let b = self.steps[j];
                if (a.x as i64) * (b.y as i64) == (a.y as i64) * (b.x as i64) {
                    return false;
                }
            }
        }
        true
    }

    fn minify_steps(&mut self, si: u32) {
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
                let last = &mut self.steps[self.n - 1];
                last.x -= v.x;
                last.y -= v.y;
            }

            let bad = !self.steps_noncollinear() || self.simpleness_index::<true>() != si;
            mem::swap(&mut self.steps_buf, &mut self.steps);
            if bad {
                break;
            }
        }
    }

    pub fn run(&mut self) {
        loop {
            self.gen();
            let si = self.simpleness_index::<false>();
            if !self.recorder.minify_more {
                if !self.recorder.contains(si)
                    && self.simpleness_index::<true>() == si
                    && self.steps_noncollinear()
                {
                    self.minify_steps(si);
                    self.recorder.insert(si, &self.steps, 0);
                }
            } else {
                self.minify_steps(si);
                let size = self.steps.iter().flat_map(|v| [v.x, v.y]).fold(0, acc_size);
                if self.simpleness_index::<true>() == si
                    && self.steps_noncollinear()
                    && !self.recorder.contains_smaller(si, size)
                {
                    let sign_sum_x: i32 = self.steps.iter().map(|v| v.x.signum()).sum();
                    if sign_sum_x < 0 {
                        self.steps.iter_mut().for_each(|v| v.x = -v.x);
                    }
                    let sign_sum_y: i32 = self.steps.iter().map(|v| v.y.signum()).sum();
                    if sign_sum_y < 0 {
                        self.steps.iter_mut().for_each(|v| v.y = -v.y);
                    }
                    self.recorder.insert(si, &self.steps, size);
                }
            }

            if !self.recorder.running.load(Ordering::SeqCst) {
                return;
            }
            self.recorder.count.fetch_add(1, Ordering::SeqCst);
        }
    }
}

fn acc_size(acc: u64, n: i32) -> u64 {
    let n_abs = n.unsigned_abs() as u64;
    acc + n_abs * n_abs * 32 + (n < 0) as u64
}
