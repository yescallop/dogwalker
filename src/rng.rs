use std::{
    cell::Cell,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    thread,
    time::{Instant, SystemTime},
};

#[derive(Debug)]
pub struct Rng(Cell<u64>);

impl Rng {
    pub fn new() -> Rng {
        let mut hasher = DefaultHasher::new();
        SystemTime::now().hash(&mut hasher);
        Instant::now().hash(&mut hasher);
        thread::current().id().hash(&mut hasher);
        Rng(hasher.finish().into())
    }

    #[inline]
    pub fn gen(&self) -> u64 {
        let mut s = self.0.get();
        s = s.wrapping_add(0xa0761d6478bd642f);
        self.0.set(s);

        let t = s as u128 * (s ^ 0xe7037ed1a0b428db) as u128;
        ((t >> 64) ^ t) as u64
    }
}
