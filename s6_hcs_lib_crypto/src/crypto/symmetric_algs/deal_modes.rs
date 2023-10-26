use std::hash::{Hash, Hasher};
use rand::random;
use crate::crypto::symmetric_algs::DEAL128;


pub enum DEALMode {
    // ECB,
    // CBC(u128),
    // CFB(u128),
    // OFB(u128),
    // CTR(u128),
    // RD(u128),
    RDH,
}


impl DEALMode {
    pub fn encrypt(&self, data: Vec<u128>, key: u128, cb: &dyn Fn(u8)) -> Vec<u128> {
        let mut b = std::collections::hash_map::DefaultHasher::new();

        let mut iv = random::<u128>() >> 1;
        let delta: u64 = iv as u64;
        data.hash(&mut b);
        let hash = b.finish() as u128;
        let deal = DEAL128::with_key(key);
        let mut enc = Vec::new();
        enc.push(deal.encrypt(iv));
        enc.push(deal.encrypt(hash ^ iv));
        let step = data.len() / 100 + 1;

        for i in 0..(data.len()) {
            if i % step == 0 {
                cb((i / step + 1) as u8);
            }
            iv += delta as u128;
            enc.push(deal.encrypt(data[i] ^ iv))
        }
        enc
    }

    pub fn decrypt(&self, data: Vec<u128>, key: u128, cb: &dyn Fn(u8)) -> Result<Vec<u128>, ()> {
        let mut b = std::collections::hash_map::DefaultHasher::new();

        let deal = DEAL128::with_key(key);
        let mut dec = Vec::new();
        let mut iv = deal.decrypt(data[0]);
        let delta: u64 = iv as u64;
        let hash = deal.decrypt(data[1]) ^ iv;
        let step = data.len() / 100 + 1;

        for i in 2..(data.len()) {
            if i % step == 0 {
                cb((i / step + 1) as u8);
            }
            iv += delta as u128;
            dec.push(deal.decrypt(data[i]) ^ iv);
        }
        dec.hash(&mut b);
        if b.finish() as u128 == hash {
            Ok(dec)
        } else {
            Err(())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_rdh() {
        let data: Vec<u128> = (0..1).map(|_| 15).collect();
        let key = DEAL128::generate_key();
        let cb = |i| { println!("{i}"); };
        let enc = DEALMode::RDH.encrypt(data.clone(), key, &cb);
        DEALMode::RDH.decrypt(enc, key, &cb).unwrap();
    }
}