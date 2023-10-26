use std::ops::{Add, Neg, Sub};
use num_bigint::{BigInt, RandBigInt};
use num_integer::Integer;
use rand::{thread_rng};
use serde::{Serialize, Deserialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct GFP2Element {
    prime: BigInt,
    pub coefficients: (BigInt, BigInt),
}


impl GFP2Element {
    pub fn get_random_coefficients(prime: BigInt) -> (BigInt, BigInt) {
        loop {
            let a = thread_rng().gen_bigint_range(
                &BigInt::from(0),
                &BigInt::from(prime.clone().sub(1))
            );
            let b = thread_rng().gen_bigint_range(
                &BigInt::from(0),
                &BigInt::from(prime.clone().sub(1))
            );
            if a.clone().ne(&b) {
                return  (a, b)
            }
        }
    }

    pub fn new(prime: BigInt) -> Self {
        let c = Self::get_random_coefficients(prime.clone());
        Self { prime, coefficients: c }
    }

    pub fn new_with_val(prime: BigInt, value: BigInt) -> Self {
        let c = value.neg().mod_floor(&prime);
        Self {
            prime: prime.clone(),
            coefficients: (c.clone(), c)
        }
    }

    pub fn new_with_coefficients(prime: BigInt, coefficients: (BigInt, BigInt)) -> Self {
        let c = (
            coefficients.0.mod_floor(&prime),
            coefficients.1.mod_floor(&prime)
        );
        Self { prime, coefficients: c }
    }

    pub fn get_swapped(&self) -> Self {
        Self::new_with_coefficients(
            self.prime.clone(),
            (
                self.coefficients.1.clone(),
                self.coefficients.0.clone()
            )
        )
    }

    pub fn get_pow_2(&self) -> Self {
        let (ref a, ref b) = self.coefficients;
        Self::new_with_coefficients(
            self.prime.clone(),
            (b * (b - 2 * a), a * (a - 2 * b))
        )
    }

    pub fn is_p1(&self) -> bool {
        self.coefficients.0.eq(&self.coefficients.1)
    }

    pub fn calc(x: &Self, y: &Self, z: &Self) -> Self {
        let (ref xa, ref xb) = x.coefficients.clone();
        let (ref ya, ref yb) = y.coefficients.clone();
        let (ref za, ref zb) = z.coefficients.clone();

        Self::new_with_coefficients(
            x.prime.clone(),
            (
                za * (ya - xb - yb) + zb * (xb - xa + yb),
                za * (xa - xb + ya) + zb * (yb - xa - ya),
            ),
        )
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        [
            self.coefficients.0.clone().to_signed_bytes_be(),
            self.coefficients.1.clone().to_signed_bytes_be()
        ].concat()
    }
}


impl PartialEq<Self> for GFP2Element {
    fn eq(&self, other: &Self) -> bool {
        self.coefficients.0.eq(&other.coefficients.0) &&
            self.coefficients.1.eq(&other.coefficients.1)
    }
}


impl Sub for GFP2Element {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let (ref xa, ref xb) = self.coefficients;
        let (ref ya, ref yb) = rhs.coefficients;
        Self::new_with_coefficients(
            self.prime.clone(),
            (xa.clone() - ya.clone(), xb.clone() - yb.clone())
        )
    }
}


impl Add for GFP2Element {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let (ref xa, ref xb) = self.coefficients;
        let (ref ya, ref yb) = rhs.coefficients;
        Self::new_with_coefficients(
            self.prime.clone(),
            (xa + ya, xb + yb)
        )
    }
}
