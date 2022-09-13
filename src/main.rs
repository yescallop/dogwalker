use std::{sync::Arc, thread};

use dogwalker::*;
use quickperm::meta::{Const, IndexPair, MetaPerm};
use rand::prelude::*;

const N: usize = 7;
const SHIFTS: u32 = usize::BITS - N.leading_zeros();
const CLOSE: bool = true;

fn main() {
    // let state = Arc::new(Max::new());
    let state = Arc::new(Collect::new(N, CLOSE).unwrap());
    for _ in 0..4 {
        let state = state.clone();
        thread::spawn(|| Simulator::new(state).run());
    }
    thread::park();
}

fn direction(i: Point<i64>, j: Point<i64>, k: Point<i64>) -> i64 {
    (k.x - i.x) * (j.y - i.y) - (j.x - i.x) * (k.y - i.y)
}

// fn on_segment(i: Point<i64>, j: Point<i64>, k: Point<i64>) -> bool {
//     i.x.min(j.x) <= k.x && k.x <= i.x.max(j.x) && i.y.min(j.y) <= k.y && k.y <= i.y.max(j.y)
// }

fn segments_intersect(p: [Point<i64>; 4]) -> bool {
    let d1 = direction(p[2], p[3], p[0]);
    let d2 = direction(p[2], p[3], p[1]);
    let d3 = direction(p[0], p[1], p[2]);
    let d4 = direction(p[0], p[1], p[3]);

    (d1 ^ d2) & (d3 ^ d4) < 0

    // if ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0)) && ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0)) {
    //     true
    // } else if d1 == 0 && on_segment(p[2], p[3], p[0]) {
    //     true
    // } else if d2 == 0 && on_segment(p[2], p[3], p[1]) {
    //     true
    // } else if d3 == 0 && on_segment(p[0], p[1], p[2]) {
    //     true
    // } else if d4 == 0 && on_segment(p[0], p[1], p[3]) {
    //     true
    // } else {
    //     false
    // }
}

struct Simulator<S: State> {
    rng: SmallRng,
    mp: MetaPerm<Const<N>>,
    steps: [Point<i32>; N],
    walk: [Point<i64>; N + 1],
    state: Arc<S>,
}

impl<S: State> Simulator<S> {
    fn new(state: Arc<S>) -> Self {
        assert!(N >= 3);
        Self {
            rng: SmallRng::from_entropy(),
            mp: MetaPerm::new_const(),
            steps: Default::default(),
            walk: Default::default(),
            state,
        }
    }

    fn gen(&mut self) {
        let mut v = Point::<i32>::default();
        let len = N - CLOSE as usize;
        for i in 0..len {
            let x = self.rng.next_u32() as i32 >> SHIFTS;
            let y = self.rng.next_u32() as i32 >> SHIFTS;
            self.steps[i] = Point { x, y };
            v.x += x;
            v.y += y;
        }
        if CLOSE {
            (v.x, v.y) = (-v.x, -v.y);
            self.steps[N - 1] = v;
        }
    }

    fn test(&mut self) -> bool {
        let mut v = Point::default();
        for i in 0..N {
            let walk = self.steps[i];
            let last_v = v;

            v.x += walk.x as i64;
            v.y += walk.y as i64;
            self.walk[i + 1] = v;

            if i >= 2 {
                let start = (CLOSE && i == N - 1) as usize;
                for j in start..i - 1 {
                    if segments_intersect([last_v, v, self.walk[j], self.walk[j + 1]]) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn count(&mut self) -> u32 {
        let mut cnt = 0;
        loop {
            cnt += self.test() as u32;
            unsafe { IndexPair::new(0, 1).swap_unchecked(&mut self.steps) }
            cnt += self.test() as u32;
            if let Some(p) = self.mp.gen_even() {
                unsafe { p.swap_unchecked(&mut self.steps) }
            } else {
                break;
            }
        }
        cnt
    }

    fn run(&mut self) {
        loop {
            self.gen();
            let cnt = self.count();
            self.state.update(cnt, &self.steps);
        }
    }
}
