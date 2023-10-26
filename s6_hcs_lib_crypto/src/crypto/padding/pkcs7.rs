use crate::crypto::padding::base::PaddingAlgorithm;

#[derive(Copy, Clone, Debug)]
pub struct PaddingPKSC7 {
    block_size: u8,
}


impl Default for PaddingPKSC7 {
    fn default() -> Self {
        Self { block_size: 32 }
    }
}


impl PaddingAlgorithm for PaddingPKSC7 {
    fn apply_padding(&self, input: &mut Vec<u8>) {
        let leftover_bytes= (input.len() % (self.block_size as usize)) as u8;
        let pad_bytes = self.block_size - leftover_bytes;

        if pad_bytes != 0 {
            input.extend(vec![pad_bytes; pad_bytes as usize]);
        } else {
            input.extend(vec![self.block_size; self.block_size as usize]);
        }
    }

    fn remove_padding(&self, input: &mut Vec<u8>) {
        input.truncate(input.len() - (input.last().unwrap().clone() as usize))
    }

    fn with_block_size(size: u8) -> Self {
        Self { block_size: size }
    }
}