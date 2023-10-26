use num_bigint::BigInt;
use crate::crypto::asymmetric_algs::{
    math::GFP2Element,
    xtr::XTRKeygen,
};


pub type PubKey = (BigInt, BigInt, GFP2Element, GFP2Element);


pub struct XTREncryptor {
    p: BigInt,
    q: BigInt,
    trace: GFP2Element,
    trace_gk: GFP2Element,
    k: Option<BigInt>,
    trace_gb: Option<GFP2Element>,
    sym_key: Option<Vec<u8>>,
}


impl XTREncryptor {
    pub fn new_at_server() -> Self {
        let mut keygen = XTRKeygen::new();
        let (p, q, trace) = keygen.generate_key();
        let (k, trace_gk) = keygen.elgamal_key();
        Self {
            p, q, trace, trace_gk, k: Some(k), trace_gb: None, sym_key: None
        }
    }

    pub fn share_public_key_with_client(&self) -> PubKey {
        (self.p.clone(), self.q.clone(), self.trace.clone(), self.trace_gk.clone())
    }

    pub fn new_at_client(pub_key: PubKey) -> Self {
        let (p, q, trace, trace_gk) = pub_key;
        let (sym_key, trace_gb) = XTRKeygen::symmetric_key(&p, &q, trace.clone(), trace_gk.clone());
        Self {
            p, q, trace, trace_gk, k: None, trace_gb: Some(trace_gb), sym_key: Some(sym_key)
        }
    }

    pub fn share_trace_with_server(&self) -> GFP2Element {
        self.trace_gb.clone().unwrap()
    }

    pub fn derive_sym_key_at_server(&mut self, trace_gb: GFP2Element) {
        self.trace_gb = Some(trace_gb.clone());
        self.sym_key = Some(XTRKeygen::symmetric_key_recall(
            self.p.clone(), self.k.clone().unwrap(), trace_gb
        ));
    }

    pub fn encrypt(&self, input: &Vec<u8>) -> Vec<u8> {
        let mut out = Vec::new();
        let key = self.sym_key.clone().unwrap();
        for i in 0..input.len() {
            out.push(input[i] ^ (key[i % key.len()]));
        }
        out
    }

    pub fn encrypt_deal128_key(&self, key: u128) -> u128 {
        let enc = self.encrypt(&(key.to_be_bytes().to_vec()));
        u128::from_be_bytes(enc.try_into().unwrap())
    }
}



#[cfg(test)]
mod tests {
    use rand::random;
    use super::*;

    #[test]
    fn test_xtr_wrapper_encrypt() {
        let mut server = XTREncryptor::new_at_server();
        let client = XTREncryptor::new_at_client(server.share_public_key_with_client());
        server.derive_sym_key_at_server(client.share_trace_with_server());

        let m1 = vec![random(), random(), random(), random()];
        let e = client.encrypt(&m1);
        let m2 = server.encrypt(&e);
        assert_eq!(m1, m2);
    }

    #[test]
    fn test_xtr_wrapper_key_encrypt() {
        let mut server = XTREncryptor::new_at_server();
        let client = XTREncryptor::new_at_client(server.share_public_key_with_client());
        server.derive_sym_key_at_server(client.share_trace_with_server());

        let m1 = random();
        let e = client.encrypt_deal128_key(m1);
        let m2 = server.encrypt_deal128_key(e);
        assert_eq!(m1, m2);
    }
}