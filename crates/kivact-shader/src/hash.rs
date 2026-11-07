//! Role: The FNV-1a 64-bit hash that identifies an assembled module.

const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// Hashes the bytes of `text` with FNV-1a, 64 bit.
pub(crate) fn fnv1a64(text: &str) -> u64 {
    text.bytes().fold(OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(PRIME)
    })
}
