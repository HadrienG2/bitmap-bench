use bit_test::{bit_const_table, bit_naive, bit_static_table};
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

pub fn criterion_benchmark(c: &mut Criterion) {
    // Bitmap which fits in the L1 cache, but whose contents and length are
    // hidden from the compiler's optimizer
    fn hidden_bitmap() -> &'static [u8] {
        static BITMAP: &[u8] = &[42u8; 32 * 1024];
        unsafe {
            std::slice::from_raw_parts(
                pessimize::hide(BITMAP.as_ptr()),
                pessimize::hide(BITMAP.len()),
            )
        }
    }

    // Query the bitmap at the same hidden location
    //
    // In this benchmark, the optimizer knows that we're accessing the same
    // bitmap in the loop but it doesn't know at which index (besides that this
    // index is resident in a CPU register at the start of the computation).
    //
    // This models a real-world complex indexing workload, where the computation
    // of the index at which the bitmap is probed is too complex for the
    // optimizer to make sense of.
    {
        let mut g = c.benchmark_group("hidden_constant");
        g.throughput(Throughput::Elements(1));
        let bitmap = hidden_bitmap();
        let index = pessimize::hide(123);
        g.bench_function("bit_naive", |b| {
            b.iter(|| pessimize::consume(bit_naive(bitmap, pessimize::hide(index))))
        });
        g.bench_function("bit_const_table", |b| {
            b.iter(|| pessimize::consume(bit_const_table(bitmap, pessimize::hide(index))))
        });
        g.bench_function("bit_static_table", |b| {
            b.iter(|| pessimize::consume(bit_static_table(bitmap, pessimize::hide(index))))
        });
    }

    // Probe each index of the bitmap linearly
    //
    // In this benchmark, the optimizer knows that we're accessing each index of
    // the bitmap in a row (and can optimize accordingly), but it does not know
    // that it's the same bitmap on each iteration of the outer benchmark loop.
    {
        let mut g = c.benchmark_group("linear");
        g.throughput(Throughput::Elements(hidden_bitmap().len() as u64));
        g.bench_function("bit_naive", |b| {
            b.iter(|| {
                let bitmap = hidden_bitmap();
                for index in 0..bitmap.len() {
                    pessimize::consume(bit_naive(bitmap, index));
                }
            })
        });
        g.bench_function("bit_const_table", |b| {
            b.iter(|| {
                let bitmap = hidden_bitmap();
                for index in 0..bitmap.len() {
                    pessimize::consume(bit_const_table(bitmap, index));
                }
            })
        });
        g.bench_function("bit_static_table", |b| {
            b.iter(|| {
                let bitmap = hidden_bitmap();
                for index in 0..bitmap.len() {
                    pessimize::consume(bit_static_table(bitmap, index));
                }
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
