#[cfg(feature = "farmhash")]
extern crate farmhash;
#[cfg(feature = "fnv")]
extern crate fnv;

extern crate xxhash_rust;

use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

use xxhash_rust::xxh3::{Xxh3, xxh3_64, xxh3_64_with_secret};

pub const XXH3_DEFAULT_SECRET_SIZE: usize = 192;

pub trait CuckooHasher: Hasher {}

pub trait CuckooBuildHasher {
    /// Type of the hasher that will be created.
    type Hasher: CuckooHasher;

    /// Creates a new hasher.
    ///
    /// Each call to `build_hasher` on the same instance should produce identical
    /// [`Hasher`]s.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::hash_map::RandomState;
    /// use std::hash::BuildHasher;
    ///
    /// let s = RandomState::new();
    /// let new_s = s.build_hasher();
    /// ```
    fn build_hasher(&self) -> Self::Hasher;

    /// Calculates the hash of a single value.
    ///
    /// This is intended as a convenience for code which *consumes* hashes, such
    /// as the implementation of a hash table or in unit tests that check
    /// whether a custom [`Hash`] implementation behaves as expected.
    ///
    /// This must not be used in any code which *creates* hashes, such as in an
    /// implementation of [`Hash`].  The way to create a combined hash of
    /// multiple values is to call [`Hash::hash`] multiple times using the same
    /// [`Hasher`], not to call this method repeatedly and combine the results.
    ///
    /// # Example
    ///
    /// ```
    /// use std::cmp::{max, min};
    /// use std::hash::{BuildHasher, Hash, Hasher};
    /// struct OrderAmbivalentPair<T: Ord>(T, T);
    /// impl<T: Ord + Hash> Hash for OrderAmbivalentPair<T> {
    ///     fn hash<H: Hasher>(&self, hasher: &mut H) {
    ///         min(&self.0, &self.1).hash(hasher);
    ///         max(&self.0, &self.1).hash(hasher);
    ///     }
    /// }
    ///
    /// // Then later, in a `#[test]` for the type...
    /// let bh = std::collections::hash_map::RandomState::new();
    /// assert_eq!(
    ///     bh.hash_one(OrderAmbivalentPair(1, 2)),
    ///     bh.hash_one(OrderAmbivalentPair(2, 1))
    /// );
    /// assert_eq!(
    ///     bh.hash_one(OrderAmbivalentPair(10, 2)),
    ///     bh.hash_one(&OrderAmbivalentPair(2, 10))
    /// );
    /// ```
    fn hash_one<T: Hash>(&self, x: T) -> u64
    where
        Self: Sized,
        Self::Hasher: Hasher,
    {
        let mut hasher = self.build_hasher();
        x.hash(&mut hasher);
        hasher.finish()
    }

    fn hash_slice(&self, x: &[u8]) -> u64 {
        let mut hasher = self.build_hasher();
        x.hash(&mut hasher);
        hasher.finish()
    }
}

// Std
impl CuckooHasher for DefaultHasher {}

#[derive(Default)]
pub struct BuildHasherStd {}
impl CuckooBuildHasher for BuildHasherStd {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        DefaultHasher::new()
    }
}

impl CuckooBuildHasher for &BuildHasherStd {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        DefaultHasher::new()
    }
}

// Xxh3
impl CuckooHasher for Xxh3 {}

pub struct BuildHasherXxh3 {
    secret: [u8; XXH3_DEFAULT_SECRET_SIZE],
}

impl BuildHasherXxh3 {
    pub fn with_secret(secret: [u8; XXH3_DEFAULT_SECRET_SIZE]) -> Self {
        Self { secret }
    }
}

impl CuckooBuildHasher for BuildHasherXxh3 {
    type Hasher = Xxh3;

    fn build_hasher(&self) -> Self::Hasher {
        Xxh3::with_secret(self.secret)
    }
    
    fn hash_slice(&self, x: &[u8]) -> u64 {
        xxh3_64_with_secret(x, &self.secret)
    }
}


#[derive(Default)]
pub struct DefaultBuildHasherXxh3 {}
impl CuckooBuildHasher for DefaultBuildHasherXxh3 {
    type Hasher = Xxh3;

    fn build_hasher(&self) -> Self::Hasher {
        Xxh3::default()
    }

    fn hash_slice(&self, x: &[u8]) -> u64 {
        xxh3_64(x)
    }
}

// Farmhash
#[cfg(feature = "farmhash")]
impl CuckooHasher for farmhash::FarmHasher {}

#[cfg(feature = "farmhash")]
#[derive(Default)]
pub struct BuildHasherFarmhash {}

#[cfg(feature = "farmhash")]
impl CuckooBuildHasher for BuildHasherFarmhash {
    type Hasher = farmhash::FarmHasher;

    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::default()
    }
}

// FNV
#[cfg(feature = "fnv")]
impl CuckooHasher for fnv::FnvHasher {}

#[cfg(feature = "fnv")]
#[derive(Default)]
pub struct BuildHasherFnv {}

#[cfg(feature = "fnv")]
impl CuckooBuildHasher for BuildHasherFnv {
    type Hasher = fnv::FnvHasher;

    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::default()
    }
}