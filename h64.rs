//! Pre-hashed 64-bit integers.

use {
    core::{fmt, hash::{BuildHasherDefault, Hasher}},
    std::collections::{HashMap, HashSet},
};

/// Pre-hashed 64-bit integer.
///
/// This stores the hash of the [`u64`] it represents,
/// eliminating the need to compute the hash on every use
/// (but only when used with [`H64Hasher`]).
/// Formatting the value reveals the original [`u64`].
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct H64(u64);

impl H64
{
    /// Create a [`H64`] from a [`u64`].
    #[inline]
    pub fn hash(mut i: u64) -> Self
    {
        i = u64::wrapping_mul(i ^ i >> 30, 0xBF58476D1CE4E5B9);
        i = u64::wrapping_mul(i ^ i >> 27, 0x94D049BB133111EB);
        i =  i ^ i >> 31;
        Self(i)
    }

    /// Obtain the original [`u64`].
    #[inline]
    pub fn unhash(self) -> u64
    {
        let Self(mut i) = self;
        i = u64::wrapping_mul(i ^ i >> 31 ^ i >> 62, 0x319642B2D24D8EC3);
        i = u64::wrapping_mul(i ^ i >> 27 ^ i >> 54, 0x96DE1B173F119089);
        i =  i ^ i >> 30 ^ i >> 60;
        i
    }
}

impl fmt::Debug for H64
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        self.unhash().fmt(f)
    }
}

impl fmt::Display for H64
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        self.unhash().fmt(f)
    }
}

/// Utility for generating [`H64`]s.
pub struct H64Allocator(u64);

impl H64Allocator
{
    /// Create a new allocator.
    #[inline(always)]
    pub fn new() -> Self
    {
        Self(0)
    }

    /// Allocate the next [`H64`].
    #[inline]
    pub fn alloc(&mut self) -> H64
    {
        let h64 = H64::hash(self.0);
        self.0 += 1;
        h64
    }
}

/// Identity hasher for use with [`H64`].
///
/// This hasher only implements [`write_u64`].
/// The implementation is the identity function.
/// Its use improves hashing performance by 100%.
///
/// [`write_u64`]: `Self::write_u64`
#[derive(Default)]
pub struct H64Hasher(u64);

impl Hasher for H64Hasher
{
    #[inline(always)]
    fn finish(&self) -> u64
    {
        self.0
    }

    fn write(&mut self, _bytes: &[u8])
    {
        unimplemented!("only use with H64")
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64)
    {
        self.0 = i;
    }
}

/// Specialization of [`HashMap`] that uses [`H64Hasher`].
pub type H64HashMap<K, V> = HashMap<K, V, BuildHasherDefault<H64Hasher>>;

/// Specialization of [`HashSet`] that uses [`H64Hasher`].
pub type H64HashSet<T> = HashSet<T, BuildHasherDefault<H64Hasher>>;

#[cfg(test)]
mod tests
{
    use {super::*, core::hash::Hash};

    fn test_values() -> impl Iterator<Item = u64>
    {
        Iterator::chain(0 .. 1000, u64::MAX - 1000 .. u64::MAX)
    }

    #[test]
    fn unhash_undos_hash()
    {
        for i in test_values() {
            assert_eq!(H64::hash(i).unhash(), i);
        }
    }

    #[test]
    fn hasher_does_nothing()
    {
        for i in test_values() {
            let h64 = H64::hash(i);
            let mut hasher = H64Hasher::default();
            Hash::hash(&h64, &mut hasher);
            assert_eq!(hasher.finish(), h64.0);
        }
    }
}
