use crate::{bucket::{Fingerprint, FINGERPRINT_SIZE}, CuckooBuildHasher};

use std::hash::{Hash, Hasher};

use byteorder::{BigEndian, WriteBytesExt};

// A struct combining *F*ingerprint *a*nd *I*ndexes,
// to have a return type with named fields
// instead of a tuple with unnamed fields.
#[derive(Copy, Clone)]
pub struct FaI {
    pub fp: Fingerprint,
    pub i1: usize,
    pub i2: usize,
}

fn get_hash<T: ?Sized + Hash, H: CuckooBuildHasher>(hash_builder: &H, data: &T) -> (u32, u32) {
    let mut hasher = hash_builder.build_hasher();
    data.hash(&mut hasher);
    let result = hasher.finish();

    // split 64bit hash value in the upper and the lower 32bit parts,
    // one used for the fingerprint, the other used for the indexes.
    ((result >> 32) as u32, result as u32)
}

fn get_slice_hash<H: CuckooBuildHasher>(hash_builder: &H, data: &[u8]) -> (u32, u32) {
    let result = hash_builder.hash_one_slice(data);

    // split 64bit hash value in the upper and the lower 32bit parts,
    // one used for the fingerprint, the other used for the indexes.
    ((result >> 32) as u32, result as u32)
}

pub fn get_alt_index<H: CuckooBuildHasher>(hash_builder: &H, fp: Fingerprint, i: usize) -> usize {
    let (_, index_hash) = get_slice_hash(hash_builder, &fp.data);
    let alt_i = index_hash as usize;
    (i ^ alt_i) as usize
}

impl FaI {
    fn from_hash<H: CuckooBuildHasher>(hash_builder: &H, fp_hash: u32, index_hash: u32) -> Self {
        let mut fp_hash_arr = [0; FINGERPRINT_SIZE];
        let _ = (&mut fp_hash_arr[..]).write_u32::<BigEndian>(fp_hash);
        let mut valid_fp_hash: [u8; FINGERPRINT_SIZE] = [0; FINGERPRINT_SIZE];
        let mut n = 0;
        let fp;

        // increment every byte of the hash until we find one that is a valid fingerprint
        loop {
            for i in 0..FINGERPRINT_SIZE {
                valid_fp_hash[i] = fp_hash_arr[i] + n;
            }

            if let Some(val) = Fingerprint::from_data(valid_fp_hash) {
                fp = val;
                break;
            }
            n += 1;
        }

        let i1 = index_hash as usize;
        let i2 = get_alt_index(hash_builder, fp, i1);
        Self { fp, i1, i2 }
    }

    fn from_data<T: ?Sized + Hash, H: CuckooBuildHasher>(hash_builder: &H, data: &T) -> Self {
        let (fp_hash, index_hash) = get_hash(hash_builder, data);
        Self::from_hash(hash_builder, fp_hash, index_hash)
    }

    fn from_slice<H: CuckooBuildHasher>(hash_builder: &H, data: &[u8]) -> Self {
        let (fp_hash, index_hash) = get_slice_hash(hash_builder, data);
        Self::from_hash(hash_builder, fp_hash, index_hash)
    }

    pub fn random_index<R: ::rand::Rng>(&self, r: &mut R) -> usize {
        if r.gen() {
            self.i1
        } else {
            self.i2
        }
    }
}

pub fn get_fai<T: ?Sized + Hash, H: CuckooBuildHasher>(hash_builder: &H, data: &T) -> FaI {
    FaI::from_data(hash_builder, data)
}

pub fn get_slice_fai<H: CuckooBuildHasher>(hash_builder: &H, data: &[u8]) -> FaI {
    FaI::from_slice(hash_builder, data)
}

#[cfg(test)]
mod tests {
    use crate::BuildHasherStd;

    use super::*;

    #[test]
    fn test_fp_and_index() {
        let build_hasher = BuildHasherStd::default();
        let data = "seif";
        let fai = get_fai(&build_hasher, data);
        let FaI { fp, i1, i2 } = fai;
        let i11 = get_alt_index(&build_hasher, fp, i2);
        assert_eq!(i11, i1);

        let i22 = get_alt_index(&build_hasher, fp, i11);
        assert_eq!(i22, i2);
    }
}
