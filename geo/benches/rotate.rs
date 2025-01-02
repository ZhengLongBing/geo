use criterion::{criterion_group, criterion_main, Criterion};
use geo::prelude::*;

fn criterion_benchmark(c: &mut Criterion) {
    // 为f32类型的旋转函数进行基准测试
    c.bench_function("rotate f32", |bencher| {
        // 获取挪威主线字符串的f32类型
        let line_string = geo_test_fixtures::norway_main::<f32>();

        bencher.iter(|| {
            // 对线字符串进行180度旋转围绕其质心
            criterion::black_box(
                criterion::black_box(&line_string)
                    .rotate_around_centroid(criterion::black_box(180.)),
            );
        });
    });

    // 为f64类型的旋转函数进行基准测试
    c.bench_function("rotate f64", |bencher| {
        // 获取挪威主线字符串的f64类型
        let line_string = geo_test_fixtures::norway_main::<f64>();

        bencher.iter(|| {
            // 对线字符串进行180度旋转围绕其质心
            criterion::black_box(
                criterion::black_box(&line_string)
                    .rotate_around_centroid(criterion::black_box(180.)),
            );
        });
    });
}

// 定义基准测试组和主函数
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
