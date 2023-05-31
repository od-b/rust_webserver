#![allow(unused_imports)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use webserver::common::fsutil::tokenize_file;
use std::time::Duration;

fn benchmark_tokenizer(c: &mut Criterion) {
    c.bench_function(
        "tokenize_enwik8",
        |b| b.iter(|| {
            tokenize_file(black_box("./data/enwik8"))
        })
    );
}

criterion_group!{
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::new(10, 0));
    targets = benchmark_tokenizer
}

criterion_main!(benches);
