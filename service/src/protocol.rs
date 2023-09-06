use std::{
    hash::{Hash, Hasher},
    collections::hash_map::DefaultHasher
};

use num_bigint::BigUint;


#[derive(Clone)]
pub struct ChaumPedersen {
    p: BigUint,
    q: BigUint,
    g: BigUint,
    h: BigUint,
}

impl ChaumPedersen {
    pub fn new(p: BigUint, q: BigUint, g: BigUint, h: BigUint) -> Self {
        Self { p, q, g, h }
    }

    pub fn register(&self, password: String) -> (BigUint, BigUint) {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let x = hasher.finish().into();
        (self.g.modpow(&x, &self.p), self.h.modpow(&x, &self.p))
    }

    pub fn commit(&self) -> (BigUint, BigUint, BigUint) {
        let k: BigUint = rand::random::<u64>().into();
        (k.clone(), self.g.modpow(&k, &self.p), self.h.modpow(&k, &self.p))
    }

    pub fn challenge(&self) -> BigUint {
        rand::random::<u64>() % &self.q
    }

    pub fn solve(&self, password: String, k: &BigUint, c: &BigUint) -> BigUint {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let x: BigUint = hasher.finish().into();

        if k >= &self.q {
            k - (c * x) % &self.q
        } else {
            &self.q + k - (c * x) % &self.q
        }
    }

    pub fn verify(&self, y1: &BigUint, y2: &BigUint, r1: &BigUint, r2: &BigUint, c: &BigUint, s: &BigUint) -> bool {
        r1 == &(self.g.modpow(s, &self.p) * y1.modpow(c, &self.p) % &self.p) &&
        r2 == &(self.h.modpow(s, &self.p) * y2.modpow(c, &self.p) % &self.p)
    }
}
