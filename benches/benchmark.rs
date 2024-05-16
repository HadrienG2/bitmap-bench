use criterion::{criterion_group, criterion_main, Criterion, Throughput};

pub fn criterion_benchmark(c: &mut Criterion) {
    // Bitmap which fits in the L1 cache, but whose contents and length are
    // hidden from the compiler's optimizer
    static BITMAP: [u8; 32 * 1024] = [42u8; 32 * 1024];
    fn with_hidden_bitmap_mut(op: impl FnOnce(&mut [u8])) {
        let mut bitmap = BITMAP;
        let hidden_bitmap = unsafe {
            std::slice::from_raw_parts_mut(
                pessimize::hide(bitmap.as_mut_ptr()),
                pessimize::hide(bitmap.len()),
            )
        };
        op(hidden_bitmap)
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
        let indices = [123, 456, 789, 1011];
        let hidden_indices = || {
            let [i1, i2, i3, i4] = indices;
            [
                pessimize::hide(i1),
                pessimize::hide(i2),
                pessimize::hide(i3),
                pessimize::hide(i4),
            ]
        };
        macro_rules! bench_check_hidden_constant {
            ($($op:ident),*) => {
                with_hidden_bitmap_mut(|bitmap| {
                    $(
                        g.bench_function(stringify!($op), |b| {
                            b.iter(|| {
                                let [i1, i2, i3, i4] = hidden_indices();
                                let [o1, o2, o3, o4] = [
                                    bit_test::$op(bitmap, i1),
                                    bit_test::$op(bitmap, i2),
                                    bit_test::$op(bitmap, i3),
                                    bit_test::$op(bitmap, i4),
                                ];
                                pessimize::consume(o1);
                                pessimize::consume(o2);
                                pessimize::consume(o3);
                                pessimize::consume(o4);
                            })
                        });
                    )*
                });
            };
        }
        bench_check_hidden_constant!(bit_test_naive, bit_test_const_table, bit_test_static_table);
        macro_rules! bench_change_hidden_constant {
            ($($op:ident),*) => {
                with_hidden_bitmap_mut(|bitmap| {
                    $(
                        g.bench_function(stringify!($op), |b| {
                            b.iter(|| {
                                let [i1, i2, i3, i4] = hidden_indices();
                                bit_test::$op(bitmap, i1);
                                bit_test::$op(bitmap, i2);
                                bit_test::$op(bitmap, i3);
                                bit_test::$op(bitmap, i4);
                                pessimize::assume_accessed(&mut bitmap.as_mut_ptr());
                            })
                        });
                    )*
                });
            };
        }
        bench_change_hidden_constant!(
            bit_set_naive,
            bit_set_const_table,
            bit_set_static_table,
            bit_clear_naive,
            bit_clear_const_table,
            bit_clear_static_table
        );
    }

    // Probe each index of the bitmap linearly
    //
    // In this benchmark, the optimizer knows that we're accessing each index of
    // the bitmap in a row (and can optimize accordingly), but it does not know
    // that it's the same bitmap on each iteration of the outer benchmark loop.
    //
    // Note that for change operations, this should optimize into a simple
    // all-ones/all-zeroes byte affectation.
    {
        let mut g = c.benchmark_group("linear_all");
        g.throughput(Throughput::Elements((BITMAP.len() * 8) as u64));
        macro_rules! bench_check_linear_all {
            ($($op:ident),*) => {
                with_hidden_bitmap_mut(|bitmap| {
                    $(
                        g.bench_function(stringify!($op), |b| {
                            b.iter(|| {
                                for byte in 0..bitmap.len() {
                                    let first_bit = byte * 8;
                                    // Here we can use 8-way loop unrolling because the input
                                    // pattern is simple and not forced to stay resident in
                                    // registers.
                                    let [o1, o2, o3, o4, o5, o6, o7, o8] = [
                                        bit_test::$op(bitmap, first_bit),
                                        bit_test::$op(bitmap, first_bit + 1),
                                        bit_test::$op(bitmap, first_bit + 2),
                                        bit_test::$op(bitmap, first_bit + 3),
                                        bit_test::$op(bitmap, first_bit + 4),
                                        bit_test::$op(bitmap, first_bit + 5),
                                        bit_test::$op(bitmap, first_bit + 6),
                                        bit_test::$op(bitmap, first_bit + 7),
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
                    )*
                });
            };
        }
        bench_check_linear_all!(bit_test_naive, bit_test_const_table, bit_test_static_table);
        macro_rules! bench_change_linear_all {
            ($($op:ident),*) => {
                with_hidden_bitmap_mut(|bitmap| {
                    $(
                        g.bench_function(stringify!($op), |b| {
                            b.iter(|| {
                                for byte in 0..bitmap.len() {
                                    let first_bit = byte * 8;
                                    // Here we can use 8-way loop unrolling because the input
                                    // pattern is simple and not forced to stay resident in
                                    // registers.
                                    bit_test::$op(bitmap, first_bit);
                                    bit_test::$op(bitmap, first_bit + 1);
                                    bit_test::$op(bitmap, first_bit + 2);
                                    bit_test::$op(bitmap, first_bit + 3);
                                    bit_test::$op(bitmap, first_bit + 4);
                                    bit_test::$op(bitmap, first_bit + 5);
                                    bit_test::$op(bitmap, first_bit + 6);
                                    bit_test::$op(bitmap, first_bit + 7);
                                    pessimize::assume_accessed(&mut bitmap.as_mut_ptr());
                                }
                            })
                        });
                    )*
                });
            };
        }
        bench_change_linear_all!(
            bit_set_naive,
            bit_set_const_table,
            bit_set_static_table,
            bit_clear_naive,
            bit_clear_const_table,
            bit_clear_static_table
        );
    }

    // Like linear_all, but uses a strided pattern so that the change operations
    // do at least require some binary arithmetic
    {
        let mut g = c.benchmark_group("linear_strided");
        g.throughput(Throughput::Elements((BITMAP.len() * 4) as u64));
        macro_rules! bench_check_linear_strided {
            ($($op:ident),*) => {
                with_hidden_bitmap_mut(|bitmap| {
                    $(
                        g.bench_function(stringify!($op), |b| {
                            b.iter(|| {
                                for byte in 0..bitmap.len() {
                                    let first_bit = byte * 8;
                                    let [o1, o2, o3, o4] = [
                                        bit_test::$op(bitmap, first_bit),
                                        bit_test::$op(bitmap, first_bit + 2),
                                        bit_test::$op(bitmap, first_bit + 4),
                                        bit_test::$op(bitmap, first_bit + 6),
                                    ];
                                    pessimize::consume(o1);
                                    pessimize::consume(o2);
                                    pessimize::consume(o3);
                                    pessimize::consume(o4);
                                }
                            })
                        });
                    )*
                });
            };
        }
        bench_check_linear_strided!(bit_test_naive, bit_test_const_table, bit_test_static_table);
        macro_rules! bench_change_linear_strided {
            ($($op:ident),*) => {
                with_hidden_bitmap_mut(|bitmap| {
                    $(
                        g.bench_function(stringify!($op), |b| {
                            b.iter(|| {
                                for byte in 0..bitmap.len() {
                                    let first_bit = byte * 8;
                                    bit_test::$op(bitmap, first_bit);
                                    bit_test::$op(bitmap, first_bit + 2);
                                    bit_test::$op(bitmap, first_bit + 4);
                                    bit_test::$op(bitmap, first_bit + 6);
                                    pessimize::assume_accessed(&mut bitmap.as_mut_ptr());
                                }
                            })
                        });
                    )*
                });
            };
        }
        bench_change_linear_strided!(
            bit_set_naive,
            bit_set_const_table,
            bit_set_static_table,
            bit_clear_naive,
            bit_clear_const_table,
            bit_clear_static_table
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
