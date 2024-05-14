#[inline]
pub fn bit_naive(bitmap: &[u8], idx: usize) -> bool {
    bitmap[idx / 8] & (1 << (idx % 8)) != 0
}

#[inline]
pub fn bit_const_table(bitmap: &[u8], idx: usize) -> bool {
    bitmap[idx >> 3] & BIT_MASK[idx & 7] != 0
}

const BIT_MASK: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

#[inline]
pub fn bit_static_table(bitmap: &[u8], idx: usize) -> bool {
    bitmap[idx >> 3] & BIT_MASK_STATIC[idx & 7] != 0
}

static BIT_MASK_STATIC: [u8; 8] = BIT_MASK;
