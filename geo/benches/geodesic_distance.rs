use criterion::{criterion_group, criterion_main};
use geo::{Distance, Geodesic};

// 定义基准测试函数
fn criterion_benchmark(c: &mut criterion::Criterion) {
    // 创建用于地球距离计算的基准测试
    c.bench_function("geodesic distance f64", |bencher| {
        let a = geo::Point::new(17.107558, 48.148636); // 定义第一个地理坐标点
        let b = geo::Point::new(16.372477, 48.208810); // 定义第二个地理坐标点

        // 在基准测试中迭代地球距离的计算
        bencher.iter(|| {
            criterion::black_box(criterion::black_box(Geodesic::distance(a, b)));
            // 计算并屏蔽距离
        });
    });
}

// 定义基准组和入口点
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
