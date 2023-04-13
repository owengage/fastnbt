use criterion::{black_box, criterion_group, criterion_main, Criterion};

use fastanvil::{complete, Region};

pub fn create_complete_chunk_by_current(c: &mut Criterion) {
    c.bench_function("chunk", |b| {
        let file = std::fs::File::open("./resources/1.19.4.mca").unwrap();

        let mut region = Region::from_stream(file).unwrap();
        let data = &region.read_chunk(0, 0).unwrap().unwrap();

        b.iter(|| {
            let chunk = complete::Chunk::from_bytes(data);
            black_box(chunk).unwrap();
        });
    });
}

criterion_group!(benches, create_complete_chunk_by_current);
criterion_main!(benches);
