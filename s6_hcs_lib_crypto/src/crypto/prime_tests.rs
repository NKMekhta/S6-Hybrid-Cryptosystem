use num_bigint::{BigInt, RandBigInt};
use num_integer::Integer;
use num_traits::{One, Zero};
use rand::thread_rng;
use std::ops::{Div, Sub};

fn extended_gcd(a: &BigInt, b: &BigInt) -> (BigInt, BigInt, BigInt) {
    let mut remainder = (a.clone(), b.clone());
    let mut s = (BigInt::one(), BigInt::zero());
    let mut t = (BigInt::zero(), BigInt::one());

    while remainder.1.ne(&BigInt::zero()) {
        if &remainder.0 < &remainder.1 {
            remainder = (remainder.1.clone(), remainder.0.clone())
        }
        let quotient = &remainder.0 / &remainder.1;
        remainder = (remainder.1.clone(), &remainder.0 - &quotient * &remainder.1);
        s = (s.1.clone(), &s.0 - &quotient * &s.1);
        t = (t.1.clone(), &t.0 - &quotient * &t.1);
    }
    return (remainder.0, s.0, t.0);
}

fn jacobi_symbol(j: (BigInt, BigInt)) -> i8 {
    let (mut a, mut n) = (j.0.clone(), j.1.clone());
    let mut result = 1;

    while a != BigInt::zero() {
        while a.is_even() {
            a >>= 1;
            let n_mod_8: BigInt = n.clone() % 8;
            if n_mod_8.eq(&BigInt::from(3)) || n_mod_8.eq(&BigInt::from(5)) {
                result = -result;
            }
        }

        std::mem::swap(&mut a, &mut n);
        if {
            a.clone().mod_floor(&BigInt::from(4)).eq(&BigInt::from(3))
                && n.clone().mod_floor(&BigInt::from(4)).eq(&BigInt::from(3))
        } {
            result = -result;
        }
        a %= n.clone();
    }

    if n == BigInt::one() {
        result
    } else {
        0
    }
}

pub fn fermat_test(n: &BigInt, i: u64) -> bool {
    if n <= &BigInt::one() || n.is_even() {
        return false;
    }
    if n <= &BigInt::from(3) {
        return true;
    }

    for _ in 0..i {
        let a = thread_rng().gen_bigint_range(&BigInt::from(2), &BigInt::from(n.sub(1)));
        if extended_gcd(&a, n).0 != BigInt::one() {
            return false;
        }
        if a.modpow(&BigInt::from(n.sub(1)), n).ne(&BigInt::one()) {
            return false;
        }
    }
    true
}

pub fn solovey_strassen_test(n: &BigInt, i: u64) -> bool {
    if n <= &BigInt::one() || n.is_even() {
        return false;
    }
    if n <= &BigInt::from(3) {
        return true;
    }

    for _ in 0..i {
        let a = thread_rng().gen_bigint_range(&BigInt::from(2), n);
        let x = jacobi_symbol((a.clone(), n.clone()));
        let tmp: BigInt = n.clone();
        let tmp: BigInt = BigInt::from(tmp.sub(1)).div(2);
        if x.is_zero() || a.modpow(&tmp, n) != BigInt::from(x) {
            return false;
        }
    }
    true
}

pub fn miller_rabin_test(n: &BigInt, i: u64) -> bool {
    if n <= &BigInt::one() || n.is_even() {
        return false;
    }
    if n <= &BigInt::from(3) {
        return true;
    }

    let mut s = 0u64;
    let mut d: BigInt = n - 1;
    while d.is_even() {
        s += 1;
        d >>= 1;
    }

    for _ in 0..i {
        let a = thread_rng().gen_bigint_range(&BigInt::from(2), &BigInt::from(n - 1));

        let mut x = a.modpow(&d, &n);
        let mut y = BigInt::zero();
        for _ in 0..s {
            y = x.modpow(&BigInt::from(2), &n);
            if y == BigInt::one() && x != BigInt::one() && x != n - 1 {
                return false;
            }
            x = y.clone()
        }
        if y != BigInt::one() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    #[test]
    fn test_extended_gcd() {
        todo!()
    }

    #[test]
    fn test_jacobi() {
        assert!(jacobi_symbol((BigInt::from(2), BigInt::from(9))) == 1);
        assert!(jacobi_symbol((BigInt::from(-2), BigInt::from(7))) == -1);
        assert!(jacobi_symbol((BigInt::from(15), BigInt::from(97))) == -1);
    }

    #[test]
    fn test_fermat() {
        assert!(fermat_test(&BigInt::from(3), 16));
        assert!(fermat_test(&BigInt::from(397), 6));
        assert!(!fermat_test(&BigInt::from(4), 100));
    }

    #[test]
    fn test_solovey_strassen() {
        assert!(solovey_strassen_test(&BigInt::from(3), 16));
        assert!(solovey_strassen_test(&BigInt::from(397), 6));
        assert!(!solovey_strassen_test(&BigInt::from(4), 100));
    }

    #[test]
    fn test_miller_rabin() {
        assert!(miller_rabin_test(&BigInt::from(3), 16));
        assert!(miller_rabin_test(&BigInt::from(397), 6));
        assert!(!miller_rabin_test(&BigInt::from(4), 100));
    }
}
