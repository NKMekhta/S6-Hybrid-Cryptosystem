use std::hash::{Hash, Hasher};
use std::sync::mpsc::{Sender};
use rand::random;
use rayon::prelude::*;
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
    pub fn encrypt(&self, input: Vec<u128>, key: u128, tx: Sender<Option<()>>) -> Vec<u128> {
        let hash = {
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            input.hash(&mut hash);
            hash.finish() as u128
        };

        let iv = random::<u128>() >> 1;
        let delta = iv as u64 as u128;
        let deal = DEAL128::with_key(key);
        let mut enc_header = vec![
            deal.encrypt(iv),
            deal.encrypt(hash ^ iv),
        ];

        let mut output: Vec<(usize, u128)> = input
            .iter()
            .enumerate()
            .collect::<Vec<(usize, &u128)>>()
            .par_iter()
            .map(|(i, &b)| {
                tx.send(Some(())).unwrap_or_default();
                (i.clone(), deal.encrypt(b ^ (iv + delta * (*i as u128 + 1))))
            })
            .collect();
        output.par_sort_unstable_by_key(|(i, _)| i.clone());
        let mut output = output
            .iter()
            .map(|(_, b)| b.clone())
            .collect();
        enc_header.append(&mut output);
        enc_header
    }


    pub fn decrypt(&self, data: Vec<u128>, key: u128, tx: Sender<Option<()>>) -> Result<Vec<u128>, ()> {
        let deal = DEAL128::with_key(key);
        let iv = deal.decrypt(data[0]);
        let delta = iv as u64 as u128;
        let in_hash = deal.decrypt(data[1]) ^ iv;

        let mut dec: Vec<(usize, u128)> = data[2..]
            .iter()
            .enumerate()
            .collect::<Vec<(usize, &u128)>>()
            .par_iter()
            .map(|(i, &b)| {
                tx.send(Some(())).unwrap_or_default();
                (i.clone(), deal.decrypt(b) ^ (iv + delta * (*i as u128 + 1)))
            })
            .collect();
        dec.par_sort_unstable_by_key(|(i, _)| i.clone());
        let dec: Vec<u128> = dec
            .iter()
            .map(|(_, b)| b.clone())
            .collect();

        let out_hash = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            dec.hash(&mut hasher);
            hasher.finish() as u128
        };

        if out_hash == in_hash {
            Ok(dec)
        } else {
            Err(())
        }
    }
}


#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread::JoinHandle;
    use super::*;


    #[test]
    fn test_rdh() {
        let data: Vec<u128> = (0..1024).map(|_| random()).collect();
        let key = DEAL128::generate_key();

        let (tx, handle) = make_progress_reporter(data.len(), |i: u8| { println!("{i}") });
        let enc = DEALMode::RDH.encrypt(data.clone(), key, tx);
        assert_eq!(enc.len(), 1024 + 2);
        handle.join().unwrap();

        let (tx, handle) = make_progress_reporter(data.len() - 2, |i: u8| { println!("{i}") });
        DEALMode::RDH.decrypt(enc, key, tx).unwrap();
        handle.join().unwrap();
    }

    fn make_progress_reporter(len: usize, cb: fn(u8)) -> (Sender<Option<()>>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            let mut cnt = 0;
            loop {
                match rx.recv() {
                    Err(_) => return,
                    Ok(None) => return,
                    Ok(Some(_)) => {
                        cnt += 1;
                        if (cnt * 100 / len) < ((cnt + 1) * 100 / len) {
                            cb((cnt * 100 / len + 1) as u8);
                        }
                        if cnt >= len {
                            return;
                        }
                    }
                }
            }
        });
        (tx, handle)
    }
}