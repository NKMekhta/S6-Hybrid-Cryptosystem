use crate::crypto::symmetric_algs::DEAL128;
use rand::random;
use rayon::prelude::*;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::Sender;

pub enum DEALMode {
    ECB,
    CBC,
    CFB,
    OFB,
    CTR,
    RD,
    RDH,
}

impl DEALMode {
    pub fn encrypt(
        &self,
        input: Vec<u128>,
        key: u128,
        tx: Option<Sender<Option<()>>>,
    ) -> Vec<u128> {
        let iv = random::<u128>() >> 1;
        let deal = DEAL128::with_key(key);

        let out = match self {

            DEALMode::ECB => {
                let mut output: Vec<(usize, u128)> = input
                    .iter()
                    .enumerate()
                    .collect::<Vec<(usize, &u128)>>()
                    .par_iter()
                    .map(|(i, &b)| {
                        if let Some(tx) = &tx {
                            tx.send(Some(())).unwrap_or_default()
                        };
                        (i.clone(), deal.encrypt(b))
                    })
                    .collect();
                output.par_sort_unstable_by_key(|(i, _)| i.clone());
                output.iter().map(|(_, b)| b.clone()).collect()
            }

            DEALMode::CBC => {
                let mut output = Vec::new();
                output.push(iv);
                for (i, b) in input.iter().enumerate() {
                    if let Some(tx) = &tx {
                        tx.send(Some(())).unwrap_or_default()
                    };
                    output.push(deal.encrypt(b ^ output[i]));
                }
                output
            }

            DEALMode::CFB => {
                let mut output = Vec::new();
                output.push(iv);
                for (i, b) in input.iter().enumerate() {
                    if let Some(tx) = &tx {
                        tx.send(Some(())).unwrap_or_default()
                    };
                    output.push(deal.encrypt(output[i]) ^ b);
                }
                output
            }

            DEALMode::OFB => {
                let mut output = vec![0; input.len() + 1];
                output[0] = iv;
                let mut last = deal.encrypt(output[0]);
                for i in 0..input.len() {
                    output[i + 1] = input[i] ^ last;
                    last = deal.encrypt(last);
                    if let Some(tx) = &tx {
                        tx.send(Some(())).unwrap_or_default()
                    };
                }
                output
            }

            DEALMode::CTR => {
                let mut output = vec![0; input.len() + 1];
                output[0] = iv;
                for i in 0..input.len() {
                    output[i + 1] = input[i] ^ deal.encrypt(iv ^ (i as u128 + 1));
                    if let Some(tx) = &tx {
                        tx.send(Some(())).unwrap_or_default()
                    };
                }
                output
            }

            DEALMode::RD => {
                let delta = iv as u64 as u128;
                let mut enc_header = vec![deal.encrypt(iv)];
                let mut output: Vec<(usize, u128)> = input
                    .iter()
                    .enumerate()
                    .collect::<Vec<(usize, &u128)>>()
                    .par_iter()
                    .map(|(i, &b)| {
                        if let Some(tx) = &tx {
                            tx.send(Some(())).unwrap_or_default()
                        };
                        (i.clone(), deal.encrypt(b ^ (iv + delta * (*i as u128 + 1))))
                    })
                    .collect();
                output.par_sort_unstable_by_key(|(i, _)| i.clone());
                let mut output = output.iter().map(|(_, b)| b.clone()).collect();
                enc_header.append(&mut output);
                enc_header
            }

            DEALMode::RDH => {
                let hash = {
                    let mut hash = std::collections::hash_map::DefaultHasher::new();
                    input.hash(&mut hash);
                    hash.finish() as u128
                };

                let delta = iv as u64 as u128;
                let mut enc_header = vec![deal.encrypt(iv), deal.encrypt(hash ^ iv)];

                let mut output: Vec<(usize, u128)> = input
                    .iter()
                    .enumerate()
                    .collect::<Vec<(usize, &u128)>>()
                    .par_iter()
                    .map(|(i, &b)| {
                        if let Some(tx) = &tx {
                            tx.send(Some(())).unwrap_or_default()
                        };
                        (i.clone(), deal.encrypt(b ^ (iv + delta * (*i as u128 + 1))))
                    })
                    .collect();
                output.par_sort_unstable_by_key(|(i, _)| i.clone());
                let mut output = output.iter().map(|(_, b)| b.clone()).collect();
                enc_header.append(&mut output);
                enc_header
            }
        };
        if let Some(tx) = &tx {
            tx.send(None).unwrap_or_default()
        };
        out
    }

    pub fn decrypt(
        &self,
        input: Vec<u128>,
        key: u128,
        tx: Option<Sender<Option<()>>>,
    ) -> Result<Vec<u128>, ()> {
        let deal = DEAL128::with_key(key);

        let out = match self {
            DEALMode::ECB => {
                let mut output: Vec<(usize, u128)> = input
                    .iter()
                    .enumerate()
                    .collect::<Vec<(usize, &u128)>>()
                    .par_iter()
                    .map(|(i, &b)| {
                        if let Some(tx) = &tx {
                            tx.send(Some(())).unwrap_or_default()
                        };
                        (i.clone(), deal.decrypt(b))
                    })
                    .collect();
                output.par_sort_unstable_by_key(|(i, _)| i.clone());
                Ok(output.iter().map(|(_, b)| b.clone()).collect())
            }

            DEALMode::CBC => {
                let mut output = Vec::new();
                for (i, &b) in input[1..].iter().enumerate() {
                    if let Some(tx) = &tx {
                        tx.send(Some(())).unwrap_or_default()
                    };
                    output.push(deal.decrypt(b) ^ input[i]);
                }
                Ok(output)
            }

            DEALMode::CFB => {
                let mut output = Vec::new();
                for (i, b) in input[1..].iter().enumerate() {
                    if let Some(tx) = &tx {
                        tx.send(Some(())).unwrap_or_default()
                    };
                    output.push(deal.encrypt(input[i]) ^ b);
                }
                Ok(output)
            }

            DEALMode::OFB => {
                let mut output = vec![0; input.len() - 1];
                let mut last = deal.encrypt(input[0]);
                for i in 1..input.len() {
                    output[i - 1] = input[i] ^ last;
                    last = deal.encrypt(last);
                    if let Some(tx) = &tx {
                        tx.send(Some(())).unwrap_or_default()
                    };
                }
                Ok(output)
            }

            DEALMode::CTR => {
                let mut output = vec![0; input.len() - 1];
                let iv = input[0];
                for i in 1..input.len() {
                    output[i - 1] = input[i] ^ deal.encrypt(iv ^ (i as u128));
                    if let Some(tx) = &tx {
                        tx.send(Some(())).unwrap_or_default()
                    };
                }
                Ok(output)
            }

            DEALMode::RD => {
                let deal = DEAL128::with_key(key);
                let iv = deal.decrypt(input[0]);
                let delta = iv as u64 as u128;
                let mut dec: Vec<(usize, u128)> = input[1..]
                    .iter()
                    .enumerate()
                    .collect::<Vec<(usize, &u128)>>()
                    .par_iter()
                    .map(|(i, &b)| {
                        if let Some(tx) = &tx {
                            tx.send(Some(())).unwrap_or_default()
                        };
                        (i.clone(), deal.decrypt(b) ^ (iv + delta * (*i as u128 + 1)))
                    })
                    .collect();
                dec.par_sort_unstable_by_key(|(i, _)| i.clone());
                let dec: Vec<u128> = dec.iter().map(|(_, b)| b.clone()).collect();
                Ok(dec)
            }

            DEALMode::RDH => {
                let deal = DEAL128::with_key(key);
                let iv = deal.decrypt(input[0]);
                let delta = iv as u64 as u128;
                let in_hash = deal.decrypt(input[1]) ^ iv;

                let mut dec: Vec<(usize, u128)> = input[2..]
                    .iter()
                    .enumerate()
                    .collect::<Vec<(usize, &u128)>>()
                    .par_iter()
                    .map(|(i, &b)| {
                        if let Some(tx) = &tx {
                            tx.send(Some(())).unwrap_or_default()
                        };
                        (i.clone(), deal.decrypt(b) ^ (iv + delta * (*i as u128 + 1)))
                    })
                    .collect();
                dec.par_sort_unstable_by_key(|(i, _)| i.clone());
                let dec: Vec<u128> = dec.iter().map(|(_, b)| b.clone()).collect();

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
        };
        if let Some(tx) = &tx {
            tx.send(None).unwrap_or_default()
        };
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all() {
        let data: Vec<u128> = (0..1024).map(|_| random()).collect();
        let key = DEAL128::generate_key();

        let enc = DEALMode::ECB.encrypt(data.clone(), key, None);
        let new_data = DEALMode::ECB.decrypt(enc, key, None).unwrap();
        assert_eq!(new_data, data);

        let enc = DEALMode::CBC.encrypt(data.clone(), key, None);
        let new_data = DEALMode::CBC.decrypt(enc, key, None).unwrap();
        assert_eq!(new_data, data);

        let enc = DEALMode::CFB.encrypt(data.clone(), key, None);
        let new_data = DEALMode::CFB.decrypt(enc, key, None).unwrap();
        assert_eq!(new_data, data);

        let enc = DEALMode::OFB.encrypt(data.clone(), key, None);
        let new_data = DEALMode::OFB.decrypt(enc, key, None).unwrap();
        assert_eq!(new_data, data);

        let enc = DEALMode::CTR.encrypt(data.clone(), key, None);
        let new_data = DEALMode::CTR.decrypt(enc, key, None).unwrap();
        assert_eq!(new_data, data);

        let enc = DEALMode::RD.encrypt(data.clone(), key, None);
        let new_data = DEALMode::RD.decrypt(enc, key, None).unwrap();
        assert_eq!(new_data, data);

        let enc = DEALMode::RDH.encrypt(data.clone(), key, None);
        let new_data = DEALMode::RDH.decrypt(enc, key, None).unwrap();
        assert_eq!(new_data, data);
    }
}
