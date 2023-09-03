use cuckoofilter::{CuckooFilter, BuildHasherStd};

#[test]
fn fingerprints() {
    let total_items = 1_000_000;

    let mut filter1 = CuckooFilter::with_capacity(BuildHasherStd::default(), total_items);
    let mut filter2 = CuckooFilter::with_capacity(BuildHasherStd::default(), total_items);
    filter1.add(&1).unwrap();
    filter1.add(&2).unwrap();
    filter2.add(&1).unwrap();
    filter2.add(&2).unwrap();
    filter2.add(&3).unwrap();

    let fp1 = filter1.fingerprint(&1);
    let fp2 = filter1.fingerprint(&2);
    let fp3 = filter1.fingerprint(&3);

    assert!(filter1.contains_fingerprint(&fp1));
    assert!(filter1.contains_fingerprint(&fp2));
    assert!(filter2.contains_fingerprint(&fp1));
    assert!(filter2.contains_fingerprint(&fp2));
    assert!(!filter1.contains_fingerprint(&fp3));
    assert!(filter2.contains_fingerprint(&fp3));

    filter1.add_fingerprint(&fp3).unwrap();

    assert!(filter1.contains(&3));
    assert!(filter2.contains_fingerprint(&fp3));

    filter1.delete_fingerprint(&fp1);
    filter2.delete_fingerprint(&fp1);

    assert!(!filter1.contains(&1));
    assert!(!filter2.contains(&1));
}
