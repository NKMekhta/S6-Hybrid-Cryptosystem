
use crate::crypto::symmetric_algs::DES;
use rand::random;

#[derive(Copy, Clone, Debug, Default)]
pub struct DEAL128 {
    round_keys: [u64; 6],
}

fn get_bit_at(i: u8) -> u64 {
    assert!((1..=64).contains(&i));
    1u64 << (64 - i)
}

fn get_round_keys(key: u128) -> [u64; 6] {
    const K: u64 = 0x_0123_4567_89ab_cdef;
    let k = ((key >> 64) as u64, key as u64);
    let mut rk = [0u64; 6];
    rk[0] = k.0;
    rk[1] = k.1 ^ rk[0];
    rk[2] = k.0 ^ rk[1] ^ get_bit_at(1);
    rk[3] = k.1 ^ rk[2] ^ get_bit_at(2);
    rk[4] = k.0 ^ rk[3] ^ get_bit_at(4);
    rk[5] = k.1 ^ rk[4] ^ get_bit_at(8);
    for i in 0..rk.len() {
        rk[i] = DES::new(K).encrypt(rk[i]);
    }
    rk
}

impl DEAL128 {
    pub fn generate_key() -> u128 {
        random()
    }

    pub fn with_key(key: u128) -> Self {
        Self {
            round_keys: get_round_keys(key),
        }
    }

    pub fn encrypt(&self, input: u128) -> u128 {
        let mut x = ((input >> 64) as u64, input as u64);
        for i in 0..6 {
            let des = DES::new(self.round_keys[i]);
            x.1 = des.encrypt(x.0) ^ x.1;
            x = (x.1, x.0);
        }
        ((x.0 as u128) << 64) | (x.1 as u128)
    }

    pub fn decrypt(&self, input: u128) -> u128 {
        let mut x = ((input >> 64) as u64, input as u64);
        for i in 0..6 {
            x = (x.1, x.0);
            let des = DES::new(self.round_keys[5 - i]);
            x.1 = des.encrypt(x.0) ^ x.1;
        }
        ((x.0 as u128) << 64) | (x.1 as u128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deal() {
        let key = DEAL128::generate_key();
        let data = random();

        let cr = DEAL128::with_key(key);
        let enc = cr.encrypt(data);
        let dec = cr.decrypt(enc);
        assert_eq!(data, dec);
    }
}
