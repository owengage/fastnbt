use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fastanvil::JavaChunk;

const CHUNK_RAW: &[u8] = include_bytes!("../resources/chunk.nbt");

pub fn fastnbt_benchmark(c: &mut Criterion) {
    c.bench_function("chunk", |b| {
        b.iter(|| {
            let chunk = JavaChunk::from_bytes(CHUNK_RAW).unwrap();
            black_box(chunk);
        });
    });
}

criterion_group!(benches, fastnbt_benchmark);
criterion_main!(benches);
