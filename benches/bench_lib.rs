extern crate xx_cuckoofilter;
#[cfg(feature = "farmhash")]
extern crate farmhash;
#[cfg(feature = "fnv")]
extern crate fnv;
extern crate rand;

use criterion::measurement::{Measurement, WallTime};
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkGroup, BenchmarkId, BatchSize};
use xx_cuckoofilter::{BuildHasherStd, DefaultBuildHasherXxh3, CuckooFilter};

use self::xx_cuckoofilter::*;
use std::convert::TryInto;
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

fn perform_insertions<H: CuckooBuildHasher + Default, M: Measurement>(g: &mut BenchmarkGroup<M>, id: BenchmarkId) {
    let num_elements = 100_000;
    let contents = get_words();
    let split: Vec<&str> = contents.split("\n").take(num_elements).collect();
    let mut cf = CuckooFilter::with_capacity(H::default(), split.len() * 2);

    g.throughput(criterion::Throughput::Elements(num_elements.try_into().unwrap()));
    g.bench_with_input(id, &split, |b, input| {
        b.iter(|| {
            cf.clear();
            for s in input {
                black_box(cf.add_slice(s.as_bytes()).ok());
            }
        });
    });
}

fn bench_new(c: &mut Criterion) {
    c.bench_function("new", |b| b.iter(|| {
        black_box(CuckooFilter::new());
    }));
}

fn bench_clear(c: &mut Criterion) {
    let num_elements = 10_000;
    let contents = get_words();
    let split: Vec<&str> = contents.split("\n").take(num_elements).collect();

    let mut cf = CuckooFilter::with_capacity(DefaultBuildHasherXxh3::default(), split.len() * 2);
    for s in split {
        black_box(cf.add_slice(s.as_bytes()).ok());
    }

    c.bench_function("clear", |b| b.iter_batched(
        || cf.clone(),
        |mut cf| black_box(cf.clear()),
        BatchSize::SmallInput,
    ));
}

fn bench_insertion(c: &mut Criterion) {
    let mut g = c.benchmark_group("Insertion");
    perform_insertions::<BuildHasherStd, WallTime>(&mut g, BenchmarkId::new("std::DefaultHasher", ""));
    perform_insertions::<DefaultBuildHasherXxh3, WallTime>(&mut g, BenchmarkId::new("xxh3", ""));
    #[cfg(feature = "fnv")]
    perform_insertions::<BuildHasherFnv, WallTime>(&mut g, BenchmarkId::new("fnv", ""));
    #[cfg(feature = "farmhash")]
    perform_insertions::<BuildHasherFarmhash, WallTime>(&mut g, BenchmarkId::new("farmhash", ""));
}

criterion_group!(benches,
    bench_new,
    bench_clear,
    bench_insertion,
);
criterion_main!(benches);