#[inline]
pub fn bit_test_naive(bitmap: &[u8], idx: usize) -> bool {
    bitmap[idx / 8] & (1 << (idx % 8)) != 0
}

#[inline]
pub fn bit_set_naive(bitmap: &mut [u8], idx: usize) {
    bitmap[idx / 8] |= 1 << (idx % 8);
}

#[inline]
pub fn bit_clear_naive(bitmap: &mut [u8], idx: usize) {
    bitmap[idx / 8] &= !(1 << (idx % 8));
}

#[inline]
pub fn bit_test_const_table(bitmap: &[u8], idx: usize) -> bool {
    bitmap[idx >> 3] & BIT_MASK[idx & 7] != 0
}

#[inline]
pub fn bit_set_const_table(bitmap: &mut [u8], idx: usize) {
    bitmap[idx >> 3] |= BIT_MASK[idx & 7]
}

#[inline]
pub fn bit_clear_const_table(bitmap: &mut [u8], idx: usize) {
    bitmap[idx >> 3] &= UNSET_BIT_MASK[idx & 7]
}

const BIT_MASK: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
const UNSET_BIT_MASK: [u8; 8] = [
    255 - 1,
    255 - 2,
    255 - 4,
    255 - 8,
    255 - 16,
    255 - 32,
    255 - 64,
    255 - 128,
];

#[inline]
pub fn bit_test_static_table(bitmap: &[u8], idx: usize) -> bool {
    bitmap[idx >> 3] & BIT_MASK_STATIC[idx & 7] != 0
}

#[inline]
pub fn bit_set_static_table(bitmap: &mut [u8], idx: usize) {
    bitmap[idx >> 3] |= BIT_MASK_STATIC[idx & 7]
}

#[inline]
pub fn bit_clear_static_table(bitmap: &mut [u8], idx: usize) {
    bitmap[idx >> 3] &= UNSET_BIT_MASK_STATIC[idx & 7]
}

static BIT_MASK_STATIC: [u8; 8] = BIT_MASK;
static UNSET_BIT_MASK_STATIC: [u8; 8] = UNSET_BIT_MASK;
