#![allow(unused_imports)]
#![allow(dead_code)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use simplesearch::common::fsutil::*;
use std::time::Duration;

fn bench_tokenizer(c: &mut Criterion) {
    c.bench_function(
        "tokenize_enwik8",
        |b| b.iter(|| {
            let fp = black_box("./static/data/enwik8");
            let words = tokenize_file::<_, Vec<_>>(fp).unwrap();
            for w in words.into_iter() {
                assert!(!w.contains(' '))
            }
        })
    );
}

fn bench_filefinder_iterative(c: &mut Criterion) {
    c.bench_function(
        "find_files: iteratively",
        |b| b.iter(|| {
            let finder = FileFinder::default();
            let dir = black_box("/Users/odin/code/");
            let query = finder.search(dir, 10000, 1000);

            if let Err(e) = query {
                panic!("{:#?}", e)
            }
        })
    );
}

fn bench_filefinder_recursive(c: &mut Criterion) {
    c.bench_function(
        "find_files: recursively",
        |b| b.iter(|| {
            let finder = FileFinder::default();
            let dir = black_box("/Users/odin/code/");
            let query = finder.search_recur(dir, 10000);

            if let Err(e) = query {
                panic!("{:#?}", e)
            }
        })
    );
}

criterion_group!{
    name = b_tokenizer;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::new(10, 0));
    targets = bench_tokenizer
}

criterion_group!{
    name = b_filefinder_rec;
    config = Criterion::default()
        .measurement_time(Duration::new(10, 0));
    targets = bench_filefinder_recursive
}

criterion_group!{
    name = b_filefinder_ite;
    config = Criterion::default()
        .measurement_time(Duration::new(10, 0));
    targets = bench_filefinder_iterative
}

// criterion_main!(b_filefinder, b_tokenizer);
criterion_main!(b_filefinder_ite, b_filefinder_rec);
