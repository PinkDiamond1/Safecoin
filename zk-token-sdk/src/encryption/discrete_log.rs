#![cfg(not(target_arch = "bpf"))]

use {
    curve25519_dalek::{ristretto::RistrettoPoint, scalar::Scalar, traits::Identity},
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[allow(dead_code)]
const TWO15: u64 = 32768;
#[allow(dead_code)]
const TWO14: u64 = 16384; // 2^14
const TWO16: u64 = 65536; // 2^16
#[allow(dead_code)]
const TWO18: u64 = 262144; // 2^18

/// Type that captures a discrete log challenge.
///
/// The goal of discrete log is to find x such that x * generator = target.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
pub struct DiscreteLog {
    /// Generator point for discrete log
    pub generator: RistrettoPoint,
    /// Target point for discrete log
    pub target: RistrettoPoint,
}

#[derive(Serialize, Deserialize, Default)]
pub struct DecodePrecomputation(HashMap<[u8; 32], u16>);

/// Builds a HashMap of 2^16 elements
#[allow(dead_code)]
fn decode_u32_precomputation(generator: RistrettoPoint) -> DecodePrecomputation {
    let mut hashmap = HashMap::new();

    let two16_scalar = Scalar::from(TWO16);
    let identity = RistrettoPoint::identity(); // 0 * G
    let generator = two16_scalar * generator; // 2^16 * G

    // iterator for 2^12*0G , 2^12*1G, 2^12*2G, ...
    let ristretto_iter = RistrettoIterator::new(identity, generator);
    ristretto_iter.zip(0..TWO16).for_each(|(elem, x_hi)| {
        let key = elem.compress().to_bytes();
        hashmap.insert(key, x_hi as u16);
    });

    DecodePrecomputation(hashmap)
}

lazy_static::lazy_static! {
    /// Pre-computed HashMap needed for decryption. The HashMap is independent of (works for) any key.
    pub static ref DECODE_PRECOMPUTATION_FOR_G: DecodePrecomputation = {
        static DECODE_PRECOMPUTATION_FOR_G_BINCODE: &[u8] =
            include_bytes!("decode_u32_precomputation_for_G.bincode");
        bincode::deserialize(DECODE_PRECOMPUTATION_FOR_G_BINCODE).unwrap_or_default()
    };
}

/// Safeves the discrete log instance using a 16/16 bit offline/online split
impl DiscreteLog {
    /// Safeves the discrete log problem under the assumption that the solution
    /// is a 32-bit number.
    pub(crate) fn decode_u32(self) -> Option<u64> {
        self.decode_online(&DECODE_PRECOMPUTATION_FOR_G, TWO16)
    }

    pub fn decode_online(self, hashmap: &DecodePrecomputation, solution_bound: u64) -> Option<u64> {
        // iterator for 0G, -1G, -2G, ...
        let ristretto_iter = RistrettoIterator::new(self.target, -self.generator);

        let mut decoded = None;
        ristretto_iter
            .zip(0..solution_bound)
            .for_each(|(elem, x_lo)| {
                let key = elem.compress().to_bytes();
                if hashmap.0.contains_key(&key) {
                    let x_hi = hashmap.0[&key];
                    decoded = Some(x_lo + solution_bound * x_hi as u64);
                }
            });
        decoded
    }
}

/// HashableRistretto iterator.
///
/// Given an initial point X and a stepping point P, the iterator iterates through
/// X + 0*P, X + 1*P, X + 2*P, X + 3*P, ...
struct RistrettoIterator {
    pub curr: RistrettoPoint,
    pub step: RistrettoPoint,
}

impl RistrettoIterator {
    fn new(curr: RistrettoPoint, step: RistrettoPoint) -> Self {
        RistrettoIterator { curr, step }
    }
}

impl Iterator for RistrettoIterator {
    type Item = RistrettoPoint;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.curr;
        self.curr += self.step;
        Some(r)
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*, curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT as G, std::time::Instant,
    };

    #[test]
    #[allow(non_snake_case)]
    fn test_serialize_decode_u32_precomputation_for_G() {
        let decode_u32_precomputation_for_G = decode_u32_precomputation(G);

        if decode_u32_precomputation_for_G.0 != DECODE_PRECOMPUTATION_FOR_G.0 {
            use std::{fs::File, io::Write, path::PathBuf};
            let mut f = File::create(PathBuf::from(
                "src/encryption/decode_u32_precomputation_for_G.bincode",
            ))
            .unwrap();
            f.write_all(&bincode::serialize(&decode_u32_precomputation_for_G).unwrap())
                .unwrap();
            panic!("Rebuild and run this test again");
        }
    }

    #[test]
    fn test_decode_correctness() {
        let amount: u64 = 65545;

        let instance = DiscreteLog {
            generator: G,
            target: Scalar::from(amount) * G,
        };

        // Very informal measurements for now
        let start_precomputation = Instant::now();
        let precomputed_hashmap = decode_u32_precomputation(G);
        let precomputation_secs = start_precomputation.elapsed().as_secs_f64();

        let start_online = Instant::now();
        let computed_amount = instance.decode_online(&precomputed_hashmap, TWO16).unwrap();
        let online_secs = start_online.elapsed().as_secs_f64();

        assert_eq!(amount, computed_amount);

        println!("16/16 Split precomputation: {:?} sec", precomputation_secs);
        println!("16/16 Split online computation: {:?} sec", online_secs);
    }
}
