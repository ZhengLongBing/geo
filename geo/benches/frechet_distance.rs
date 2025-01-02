use criterion::{criterion_group, criterion_main};
use geo::frechet_distance::FrechetDistance;

// 定义基准测试函数
fn criterion_benchmark(c: &mut criterion::Criterion) {
    // 创建 f32 类型的欧氏距离基准测试
    c.bench_function("frechet distance f32", |bencher| {
        let ls_a = geo_test_fixtures::vw_orig::<f32>(); // 获取原始线串数据
        let ls_b = geo_test_fixtures::vw_simplified::<f32>(); // 获取简化后线串数据

        // 在基准测试迭代中计算离散Fréchet距离
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls_a).frechet_distance(criterion::black_box(&ls_b)),
            );
        });
    });

    // 创建 f64 类型的欧氏距离基准测试
    c.bench_function("frechet distance f64", |bencher| {
        let ls_a = geo_test_fixtures::vw_orig::<f64>(); // 获取原始线串数据
        let ls_b = geo_test_fixtures::vw_simplified::<f64>(); // 获取简化后线串数据

        // 在基准测试迭代中计算离散Fréchet距离
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls_a).frechet_distance(criterion::black_box(&ls_b)),
            );
        });
    });
}

// 定义基准组和入口点
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
