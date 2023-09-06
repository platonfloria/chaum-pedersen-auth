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

    pub fn register(&self, password: &str) -> (BigUint, BigUint) {
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

    pub fn solve(&self, password: &str, k: &BigUint, c: &BigUint) -> BigUint {
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


#[cfg(test)]
mod tests {
    use super::*;

    pub fn setup_protocol() -> ChaumPedersen {
        ChaumPedersen::new(
            BigUint::from(363967321904221003u64),
            BigUint::from(7696033u64),
            BigUint::from(165950041202038920u64),
            BigUint::from(96429580695728554u64),
        )
    }

    #[test]
    fn test_register() {
        let protocol = setup_protocol();
        let (y1, y2) = protocol.register("password");
        assert_eq!(y1, BigUint::from(180020373440730202u64));
        assert_eq!(y2, BigUint::from(138713557362284185u64));
    }

    #[test]
    fn test_commit() {
        let protocol = setup_protocol();
        let (k, r1, r2) = protocol.commit();
        assert_eq!(r1, BigUint::from(165950041202038920u64).modpow(&k, &BigUint::from(363967321904221003u64)));
        assert_eq!(r2, BigUint::from(96429580695728554u64).modpow(&k, &BigUint::from(363967321904221003u64)));
    }

    #[test]
    fn test_challenge() {
        let protocol = setup_protocol();
        let c = protocol.challenge();
        assert!(c < BigUint::from(7696033u64));
    }

    #[test]
    fn test_solve() {
        let protocol = setup_protocol();
        let s = protocol.solve("password", &BigUint::from(9223918093844043694u64), &BigUint::from(4051888u64));
        assert_eq!(s, BigUint::from(9223918093840913154u64));
    }

    #[test]
    fn test_verify() {
        let protocol = setup_protocol();
        assert!(protocol.verify(
            &BigUint::from(180020373440730202u64),
            &BigUint::from(138713557362284185u64),
            &BigUint::from(254414293247193407u64),
            &BigUint::from(320950112331669597u64),
            &BigUint::from(4051888u64),
            &BigUint::from(9223918093840913154u64)
        ));
    }

    #[test]
    fn test_verify_fails() {
        let protocol = setup_protocol();
        assert!(!protocol.verify(
            &BigUint::from(180020373440730202u64),
            &BigUint::from(138713557362284185u64),
            &BigUint::from(254414293247193407u64),
            &BigUint::from(320950112331669597u64),
            &BigUint::from(4051888u64),
            &BigUint::from(1337u64)
        ));
    }
}
