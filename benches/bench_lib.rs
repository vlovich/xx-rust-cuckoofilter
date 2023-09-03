#![feature(test)]

extern crate cuckoofilter;
#[cfg(feature = "farmhash")]
extern crate farmhash;
#[cfg(feature = "fnv")]
extern crate fnv;
extern crate rand;
extern crate test;

use self::cuckoofilter::*;
use std::fs::File;
use std::hint::black_box;
use std::io::prelude::*;
use std::path::Path;

fn get_words() -> String {
    let path = Path::new("/usr/share/dict/words");
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut contents = String::new();
    if let Err(why) = file.read_to_string(&mut contents) {
        panic!("couldn't read {}: {}", display, why);
    }
    contents
}

fn perform_insertions<H: CuckooBuildHasher + Default>(b: &mut test::Bencher) {
    let contents = get_words();
    let split: Vec<&str> = contents.split("\n").take(10000).collect();
    let mut cf = CuckooFilter::with_capacity(H::default(), split.len() * 2);

    b.iter(|| {
        cf.clear();
        for s in &split {
            black_box(cf.add_slice(s.as_bytes()).ok());
        }
    });
}

#[bench]
fn bench_new(b: &mut test::Bencher) {
    b.iter(|| {
        black_box(CuckooFilter::new());
    });
}

#[bench]
fn bench_clear(b: &mut test::Bencher) {
    let mut cf = black_box(CuckooFilter::new());

    b.iter(|| {
        black_box(cf.clear());
    });
}

#[cfg(feature = "farmhash")]
#[bench]
fn bench_insertion_farmhash(b: &mut test::Bencher) {
    perform_insertions::<BuildHasherFarmhash>(b);
}

#[cfg(feature = "fnv")]
#[bench]
fn bench_insertion_fnv(b: &mut test::Bencher) {
    perform_insertions::<BuildHasherFnv>(b);
}

#[bench]
fn bench_insertion_default(b: &mut test::Bencher) {
    perform_insertions::<BuildHasherStd>(b);
}

#[bench]
fn bench_insertion_xxh3(b: &mut test::Bencher) {
    perform_insertions::<DefaultBuildHasherXxh3>(b);
}
