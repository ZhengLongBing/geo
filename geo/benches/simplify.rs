use criterion::{criterion_group, criterion_main, Criterion};
use geo::simplify::Simplify;

// 定义基准测试函数
fn criterion_benchmark(c: &mut Criterion) {
    // 对f32类型的简化算法进行基准测试
    c.bench_function("simplify simple f32", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>(); // 获取f32类型的路易斯安那线字符串
        bencher.iter(|| {
            // 对线字符串进行简化，简化至0.01的阈值
            criterion::black_box(criterion::black_box(&ls).simplify(criterion::black_box(&0.01)));
        });
    });

    // 对f64类型的简化算法进行基准测试
    c.bench_function("simplify simple f64", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f64>(); // 获取f64类型的路易斯安那线字符串
        bencher.iter(|| {
            // 对线字符串进行简化，简化至0.01的阈值
            criterion::black_box(criterion::black_box(&ls).simplify(criterion::black_box(&0.01)));
        });
    });
}

// 定义基准测试组和主函数
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
