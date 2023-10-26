mod base;
mod pkcs7;

pub use base::PaddingAlgorithm;
pub use pkcs7::PaddingPKSC7;



// Convert Vec<u128> to Vec<u8>
#[cfg(test)]
mod tests {
    use crate::crypto::padding::base::PaddingAlgorithm;
    use super::*;

    #[test]
    fn test_pkcs7() {
        let padder = PaddingPKSC7::with_block_size(8);
        let data = vec![
            vec![5u8; 8],
            vec![4u8; 5],
            vec![4u8; 163],
        ];
        for d in data {
            let mut r = d.clone();
            padder.apply_padding(&mut r);
            padder.remove_padding(&mut r);
            assert_eq!(d, r);
        }
    }
}

