pub trait PaddingAlgorithm
where
    Self: Copy
{
    fn apply_padding(&self, input: &mut Vec<u8>);
    fn remove_padding(&self, input: &mut Vec<u8>);

    fn with_block_size(size: u8) -> Self;
}
