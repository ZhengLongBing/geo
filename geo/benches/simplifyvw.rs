use criterion::{criterion_group, criterion_main, Criterion};
use geo::prelude::*;
use geo::simplify_vw::SimplifyVwPreserve;

// 定义基准测试函数
fn criterion_benchmark(c: &mut Criterion) {
    // 为 f32 精度的简化（vw 方法）进行基准测试
    c.bench_function("simplify vw simple f32", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls).simplify_vw(criterion::black_box(&0.0005)), // 应用简化函数
            );
        });
    });

    // 为 f64 精度的简化（vw 方法）进行基准测试
    c.bench_function("simplify vw simple f64", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f64>();
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls).simplify_vw(criterion::black_box(&0.0005)), // 应用简化函数
            );
        });
    });

    // 为 f32 精度的简化（vw 保留方法）进行基准测试
    c.bench_function("simplify vwp f32", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls).simplify_vw_preserve(criterion::black_box(&0.0005)), // 应用保留简化函数
            );
        });
    });

    // 为 f64 精度的简化（vw 保留方法）进行基准测试
    c.bench_function("simplify vwp f64", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f64>();
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls).simplify_vw_preserve(criterion::black_box(&0.0005)), // 应用保留简化函数
            );
        });
    });
}

// 定义基准测试组和主函数
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
