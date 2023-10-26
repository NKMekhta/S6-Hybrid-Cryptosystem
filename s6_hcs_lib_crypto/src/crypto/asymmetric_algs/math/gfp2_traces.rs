use crate::crypto::asymmetric_algs::math::GFP2Element;
use maplit::hashmap;
use num_bigint::BigInt;
use num_traits::{One, Zero};
use std::collections::HashMap;
use std::ops::Shl;

pub struct GFP2Traces {
    prime: BigInt,
    c: Option<GFP2Element>,
    map: HashMap<BigInt, GFP2Element>,
}

impl GFP2Traces {
    pub fn new(prime: BigInt) -> Self {
        Self {
            prime,
            c: None,
            map: HashMap::new(),
        }
    }

    pub fn calc_trace(&mut self, n: BigInt, c: Option<GFP2Element>) -> GFP2Element {
        if let Some(nc) = c {
            if {
                if let Some(sc) = self.c.clone() {
                    sc != nc
                } else {
                    true
                }
            } {
                self.c = Some(nc);
                self.map = hashmap! {
                    BigInt::zero() => GFP2Element::new_with_val(
                        self.prime.clone(),
                        BigInt::from(3)
                    ),
                    BigInt::one() => self.c.clone().unwrap(),
                };
            }
        }
        self.calc_s(n)
    }

    fn calc_s(&mut self, n: BigInt) -> GFP2Element {
        if let Some(a) = self.map.get(&n) {
            return a.clone();
        }

        let mut n_curr = BigInt::one();
        for i in 1..n.bits() {
            let bit = n.bit(n.bits() - i - 1);
            let n_new: BigInt = n_curr.clone().shl(1) + BigInt::from(bit);
            if !self.map.contains_key(&n_new) {
                let c_curr = self.map.get(&n_curr).unwrap().clone();
                let to_insert = if bit {
                    GFP2Element::calc(
                        &self.calc_s(n_curr.clone() + 1),
                        &self.c.clone().unwrap(),
                        &c_curr,
                    ) + self.calc_s(n_curr.clone() - 1).get_swapped()
                } else {
                    c_curr.get_pow_2() - (c_curr.clone() + c_curr).get_swapped()
                };
                self.map.insert(n_new.clone(), to_insert);
            }
            n_curr = n_new;
        }
        self.map.get(&n).clone().unwrap().clone()
    }
}
