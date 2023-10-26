use std::ops::{Add, Sub};
use num_bigint::{BigInt, RandBigInt};
use num_integer::Integer;
use num_traits::One;
use rand::thread_rng;

use crate::crypto::{
    prime_tests::miller_rabin_test,
    asymmetric_algs::math::{
        GFP2Element,
        GFP2Traces
    },
};


pub type PubKey = (BigInt, BigInt, GFP2Element);
pub type SecretKey = (BigInt, GFP2Element);
pub type SymmetricKey = (Vec<u8>, GFP2Element);


pub struct XTRKeygen {
    pub_key: Option<PubKey>,
}


impl XTRKeygen {
    pub fn new() -> XTRKeygen {
        Self {
            pub_key: None
        }
    }

    pub fn generate_key(&mut self) -> PubKey {
        let (ref r, ref q) = loop {
            let r = thread_rng().gen_bigint_range(
                &BigInt::from(u128::MIN),
                &BigInt::from(u128::MAX),
            );
            let q: BigInt = r.clone().pow(2) - r.clone() + 1;
            if {
                q.clone().mod_floor(&BigInt::from(12)).eq(&BigInt::from(7))
                && miller_rabin_test(&q, 1024)
            } {
                break (r, q)
            }
        };

        let ref p = loop {
            let k = thread_rng().gen_bigint_range(
                &BigInt::from(u128::MIN),
                &BigInt::from(u128::MAX),
            );
            let p: BigInt = r + k * q;
            if {
                p.clone().mod_floor(&BigInt::from(3)).eq(&BigInt::from(2))
                && miller_rabin_test(&p, 1024)
            } {
                break p
            }
        };

        let quotient = p
            .pow(2)
            .add(BigInt::one())
            .sub(p)
            .div_floor(q);
        let three = GFP2Element::new_with_val(
            p.clone(),
            BigInt::from(3)
        );

        let mut tr = GFP2Traces::new(p.clone());
        let trace = loop {
            let c = GFP2Element::new(p.clone());
            if tr.calc_trace(p + 1, Some(c.clone())).is_p1() {
                let trace = tr.calc_trace(quotient.clone(), None);
                if trace != three {
                    break trace
                }
            }
        };

        self.pub_key = Some((p.clone(), q.clone(), trace));
        self.pub_key.clone().unwrap()
    }

    pub fn elgamal_key(&self) -> SecretKey {
        let pub_key = self.pub_key.clone().unwrap();
        let mut tr = GFP2Traces::new(pub_key.0.clone());
        let k = thread_rng().gen_bigint_range(
            &BigInt::from(2),
            &pub_key.1.sub(3),
        );
        let c = GFP2Element::new_with_coefficients(
            pub_key.0.clone(), pub_key.2.coefficients.clone()
        );
        let trace_gk = tr.calc_trace(k.clone(),Some(c));
        (k, trace_gk)
    }

    pub fn symmetric_key(p: &BigInt, q: &BigInt, trace: GFP2Element, trace_k: GFP2Element) -> SymmetricKey {
        let b = thread_rng().gen_bigint_range(
            &BigInt::from(2),
            &q.sub(3),
        );
        let mut tr = GFP2Traces::new(p.clone());

        let c = GFP2Element::new_with_coefficients(
            p.clone(),
            trace.coefficients.clone()
        );
        let trace_gb = tr.calc_trace(b.clone(), Some(c));

        let c = GFP2Element::new_with_coefficients(
            p.clone(),
            trace_k.coefficients.clone()
        );
        let trace_gbk = tr.calc_trace(b, Some(c));
        (trace_gbk.get_bytes(), trace_gb)
    }

    pub fn symmetric_key_recall(p: BigInt, k: BigInt, trace_gb: GFP2Element) -> Vec<u8> {
        GFP2Traces::new(p.clone())
            .calc_trace(
                k,
                Some(GFP2Element::new_with_coefficients(
                    p.clone(),
                    trace_gb.coefficients.clone(),
                ))
            ).get_bytes()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xtr() {
        for _ in 0..4 {
            // server (alice)
            let mut xtr = XTRKeygen::new();
            let (p, q, trace) = xtr.generate_key();
            let (k, trace_gk) = xtr.elgamal_key();

            // client (bob)
            let (ck, trace_gb) = XTRKeygen::symmetric_key(&p, &q, trace.clone(), trace_gk.clone());

            // server
            let sk = XTRKeygen::symmetric_key_recall(p.clone(), k.clone(), trace_gb.clone());
            assert_eq!(sk, ck);
        }
    }
}