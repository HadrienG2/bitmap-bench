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

    // Query the bitmap at the same hidden locations
    //
    // In this benchmark, the optimizer knows that we're accessing the same
    // bitmap in the loop but it doesn't know at which indices (besides the fact
    // that each index will be resident in a general-purpose CPU register at the
    // start of the computation).
    //
    // This models a real-world complex indexing workload, where the computation
    // of the index at which the bitmap is probed is too complex for the
    // optimizer to make sense of.
    {
        // Loop unrolling factor used to amortize benchmark harness and
        // optimization barrier overheads. Should not be greater than 4 because
        // x86_64 has 16 registers and we need UNROLL_FACTOR of them for inputs
        // and UNROLL_FACTOR for outputs.
        const UNROLL_FACTOR: usize = 4;
        let mut g = c.benchmark_group("hidden_constant");
        g.throughput(Throughput::Elements(UNROLL_FACTOR as u64));
        let bitmap = hidden_bitmap();
        let indices = [123, 456, 789, 1011];
        assert_eq!(indices.len(), UNROLL_FACTOR);
        g.bench_function("bit_naive", |b| {
            b.iter(|| {
                let [i1, i2, i3, i4] = indices;
                let [o1, o2, o3, o4] = [
                    bit_naive(bitmap, pessimize::hide(i1)),
                    bit_naive(bitmap, pessimize::hide(i2)),
                    bit_naive(bitmap, pessimize::hide(i3)),
                    bit_naive(bitmap, pessimize::hide(i4)),
                ];
                pessimize::consume(o1);
                pessimize::consume(o2);
                pessimize::consume(o3);
                pessimize::consume(o4);
            })
        });
        g.bench_function("bit_const_table", |b| {
            b.iter(|| {
                let [i1, i2, i3, i4] = indices;
                let [o1, o2, o3, o4] = [
                    bit_const_table(bitmap, pessimize::hide(i1)),
                    bit_const_table(bitmap, pessimize::hide(i2)),
                    bit_const_table(bitmap, pessimize::hide(i3)),
                    bit_const_table(bitmap, pessimize::hide(i4)),
                ];
                pessimize::consume(o1);
                pessimize::consume(o2);
                pessimize::consume(o3);
                pessimize::consume(o4);
            })
        });
        g.bench_function("bit_static_table", |b| {
            b.iter(|| {
                let [i1, i2, i3, i4] = indices;
                let [o1, o2, o3, o4] = [
                    bit_static_table(bitmap, pessimize::hide(i1)),
                    bit_static_table(bitmap, pessimize::hide(i2)),
                    bit_static_table(bitmap, pessimize::hide(i3)),
                    bit_static_table(bitmap, pessimize::hide(i4)),
                ];
                pessimize::consume(o1);
                pessimize::consume(o2);
                pessimize::consume(o3);
                pessimize::consume(o4);
            })
        });
    }

    // Probe each index of the bitmap linearly
    //
    // In this benchmark, the optimizer knows that we're accessing each index of
    // the bitmap in a row (and can optimize accordingly), but it does not know
    // that it's the same bitmap on each iteration of the outer benchmark loop.
    {
        let mut g = c.benchmark_group("linear");
        g.throughput(Throughput::Elements((hidden_bitmap().len() * 8) as u64));
        g.bench_function("bit_naive", |b| {
            b.iter(|| {
                let bitmap = hidden_bitmap();
                for byte in 0..bitmap.len() {
                    let first_bit = byte * 8;
                    // Here we can use 8-way loop unrolling because the input
                    // pattern is simple and not forced to stay resident in
                    // registers.
                    let [o1, o2, o3, o4, o5, o6, o7, o8] = [
                        bit_naive(bitmap, first_bit),
                        bit_naive(bitmap, first_bit + 1),
                        bit_naive(bitmap, first_bit + 2),
                        bit_naive(bitmap, first_bit + 3),
                        bit_naive(bitmap, first_bit + 4),
                        bit_naive(bitmap, first_bit + 5),
                        bit_naive(bitmap, first_bit + 6),
                        bit_naive(bitmap, first_bit + 7),
                    ];
                    pessimize::consume(o1);
                    pessimize::consume(o2);
                    pessimize::consume(o3);
                    pessimize::consume(o4);
                    pessimize::consume(o5);
                    pessimize::consume(o6);
                    pessimize::consume(o7);
                    pessimize::consume(o8);
                }
            })
        });
        g.bench_function("bit_const_table", |b| {
            b.iter(|| {
                let bitmap = hidden_bitmap();
                for byte in 0..bitmap.len() {
                    let first_bit = byte * 8;
                    // Here we can use 8-way loop unrolling because the input
                    // pattern is simple and not forced to stay resident in
                    // registers.
                    let [o1, o2, o3, o4, o5, o6, o7, o8] = [
                        bit_const_table(bitmap, first_bit),
                        bit_const_table(bitmap, first_bit + 1),
                        bit_const_table(bitmap, first_bit + 2),
                        bit_const_table(bitmap, first_bit + 3),
                        bit_const_table(bitmap, first_bit + 4),
                        bit_const_table(bitmap, first_bit + 5),
                        bit_const_table(bitmap, first_bit + 6),
                        bit_const_table(bitmap, first_bit + 7),
                    ];
                    pessimize::consume(o1);
                    pessimize::consume(o2);
                    pessimize::consume(o3);
                    pessimize::consume(o4);
                    pessimize::consume(o5);
                    pessimize::consume(o6);
                    pessimize::consume(o7);
                    pessimize::consume(o8);
                }
            })
        });
        g.bench_function("bit_static_table", |b| {
            b.iter(|| {
                let bitmap = hidden_bitmap();
                for byte in 0..bitmap.len() {
                    let first_bit = byte * 8;
                    // Here we can use 8-way loop unrolling because the input
                    // pattern is simple and not forced to stay resident in
                    // registers.
                    let [o1, o2, o3, o4, o5, o6, o7, o8] = [
                        bit_static_table(bitmap, first_bit),
                        bit_static_table(bitmap, first_bit + 1),
                        bit_static_table(bitmap, first_bit + 2),
                        bit_static_table(bitmap, first_bit + 3),
                        bit_static_table(bitmap, first_bit + 4),
                        bit_static_table(bitmap, first_bit + 5),
                        bit_static_table(bitmap, first_bit + 6),
                        bit_static_table(bitmap, first_bit + 7),
                    ];
                    pessimize::consume(o1);
                    pessimize::consume(o2);
                    pessimize::consume(o3);
                    pessimize::consume(o4);
                    pessimize::consume(o5);
                    pessimize::consume(o6);
                    pessimize::consume(o7);
                    pessimize::consume(o8);
                }
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
