use std::{
    hash::{Hash, Hasher},
    collections::hash_map::DefaultHasher
};

use k256::{
    AffinePoint,
    Scalar,
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


#[derive(Clone)]
pub struct ChaumPedersenK256 {
    g: AffinePoint,
    h: AffinePoint,
}

impl ChaumPedersenK256 {
    pub fn new(h_offset: u64) -> Self {
        Self {
            g: AffinePoint::GENERATOR,
            h: AffinePoint::from(AffinePoint::GENERATOR * Scalar::from(h_offset)),
        }
    }

    pub fn register(&self, password: &str) -> (AffinePoint, AffinePoint) {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let x = Scalar::from(hasher.finish());
        (AffinePoint::from(self.g * x), AffinePoint::from(self.h * x))
    }

    pub fn commit(&self) -> (Scalar, AffinePoint, AffinePoint) {
        let k = Scalar::from(rand::random::<u64>());
        (k.clone(), AffinePoint::from(self.g * k), AffinePoint::from(self.h * k))
    }

    pub fn challenge(&self) -> Scalar {
        Scalar::from(rand::random::<u64>())
    }

    pub fn solve(&self, password: &str, k: &Scalar, c: &Scalar) -> Scalar {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        let x = Scalar::from(hasher.finish());
        *k - c * &x
    }

    pub fn verify(&self, y1: &AffinePoint, y2: &AffinePoint, r1: &AffinePoint, r2: &AffinePoint, c: &Scalar, s: &Scalar) -> bool {
        *r1 == self.g * s + *y1 * *c &&
        *r2 == self.h * s + *y2 * *c
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    mod exponent {
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
                &BigUint::from(9223918093840913154u64),
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
                &BigUint::from(1337u64),
            ));
        }
    }

    mod k256 {
        use ::k256::{
            elliptic_curve::{subtle::Choice, PrimeField},
            elliptic_curve::point::DecompressPoint,
        };

        use super::*;

        pub fn setup_protocol() -> ChaumPedersenK256 {
            ChaumPedersenK256::new(107211496160805127)
        }

        #[test]
        fn test_register() {
            let protocol = setup_protocol();
            let (y1, y2) = protocol.register("password");
            assert_eq!(y1, AffinePoint::decompress(
                &[187, 39, 77, 94, 91, 203, 139, 150, 167, 164, 62, 83, 208, 74, 209, 123, 172, 90, 139, 251, 210, 68, 183, 121, 174, 51, 126, 104, 44, 106, 161, 185].into(),
                Choice::from(1),
            ).unwrap());
            assert_eq!(y2, AffinePoint::decompress(
                &[195, 56, 233, 7, 21, 30, 207, 163, 175, 82, 207, 135, 51, 182, 124, 218, 193, 110, 212, 26, 163, 42, 91, 11, 192, 158, 228, 233, 74, 169, 125, 87].into(),
                Choice::from(0)
            ).unwrap());
        }

        #[test]
        fn test_commit() {
            let protocol = setup_protocol();
            let (k, r1, r2) = protocol.commit();
            assert_eq!(r1, AffinePoint::from(AffinePoint::GENERATOR * k));
            assert_eq!(r2, AffinePoint::from(AffinePoint::from(AffinePoint::GENERATOR * Scalar::from(107211496160805127u64)) * k));
        }

        #[test]
        fn test_challenge() {
            let protocol = setup_protocol();
            protocol.challenge();
        }

        #[test]
        fn test_solve() {
            let protocol = setup_protocol();
            let s = protocol.solve(
                "password",
                &Scalar::from_repr([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 122, 89, 221, 54, 45, 103, 228, 32].into()).unwrap(),
                &Scalar::from_repr([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 166, 184, 114, 226, 167, 121, 136, 254].into()).unwrap(),
            );
            assert_eq!(s, Scalar::from_repr([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 75, 224, 144, 209, 44, 157, 92, 155, 237, 25, 142, 143, 185, 165, 204, 141].into()).unwrap());
        }

        #[test]
        fn test_verify() {
            let protocol = setup_protocol();
            assert!(protocol.verify(
                &AffinePoint::decompress(
                    &[187, 39, 77, 94, 91, 203, 139, 150, 167, 164, 62, 83, 208, 74, 209, 123, 172, 90, 139, 251, 210, 68, 183, 121, 174, 51, 126, 104, 44, 106, 161, 185].into(),
                    Choice::from(1)
                ).unwrap(),
                &AffinePoint::decompress(
                    &[195, 56, 233, 7, 21, 30, 207, 163, 175, 82, 207, 135, 51, 182, 124, 218, 193, 110, 212, 26, 163, 42, 91, 11, 192, 158, 228, 233, 74, 169, 125, 87].into(),
                    Choice::from(0)
                ).unwrap(),
                &AffinePoint::decompress(
                    &[243, 69, 153, 44, 17, 5, 189, 149, 66, 129, 164, 182, 208, 47, 6, 102, 237, 102, 177, 174, 208, 95, 213, 70, 136, 61, 163, 1, 28, 198, 125, 158].into(),
                    Choice::from(1)
                ).unwrap(),
                &AffinePoint::decompress(
                    &[105, 24, 234, 88, 96, 244, 223, 122, 159, 252, 101, 34, 217, 107, 241, 62, 195, 13, 228, 61, 132, 230, 216, 58, 114, 5, 166, 88, 45, 0, 79, 10].into(),
                    Choice::from(1)
                ).unwrap(),
                &Scalar::from_repr([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 166, 184, 114, 226, 167, 121, 136, 254].into()).unwrap(),
                &Scalar::from_repr([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 75, 224, 144, 209, 44, 157, 92, 155, 237, 25, 142, 143, 185, 165, 204, 141].into()).unwrap(),
            ));
        }

        #[test]
        fn test_verify_fails() {
            let protocol = setup_protocol();
            assert!(!protocol.verify(
                &AffinePoint::decompress(
                    &[187, 39, 77, 94, 91, 203, 139, 150, 167, 164, 62, 83, 208, 74, 209, 123, 172, 90, 139, 251, 210, 68, 183, 121, 174, 51, 126, 104, 44, 106, 161, 185].into(),
                    Choice::from(1)
                ).unwrap(),
                &AffinePoint::decompress(
                    &[195, 56, 233, 7, 21, 30, 207, 163, 175, 82, 207, 135, 51, 182, 124, 218, 193, 110, 212, 26, 163, 42, 91, 11, 192, 158, 228, 233, 74, 169, 125, 87].into(),
                    Choice::from(0)
                ).unwrap(),
                &AffinePoint::decompress(
                    &[243, 69, 153, 44, 17, 5, 189, 149, 66, 129, 164, 182, 208, 47, 6, 102, 237, 102, 177, 174, 208, 95, 213, 70, 136, 61, 163, 1, 28, 198, 125, 158].into(),
                    Choice::from(1)
                ).unwrap(),
                &AffinePoint::decompress(
                    &[105, 24, 234, 88, 96, 244, 223, 122, 159, 252, 101, 34, 217, 107, 241, 62, 195, 13, 228, 61, 132, 230, 216, 58, 114, 5, 166, 88, 45, 0, 79, 10].into(),
                    Choice::from(1)
                ).unwrap(),
                &Scalar::from_repr([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 166, 184, 114, 226, 167, 121, 136, 254].into()).unwrap(),
                &Scalar::from_repr([254, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254, 75, 224, 144, 209, 44, 157, 92, 155, 237, 25, 142, 143, 185, 165, 204, 141].into()).unwrap(),
            ));
        }
    }
}
