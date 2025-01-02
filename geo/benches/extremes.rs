use criterion::{criterion_group, criterion_main, Criterion};
use geo::prelude::*;
use geo::Polygon;

// 定义基准测试函数
fn criterion_benchmark(c: &mut Criterion) {
    // 为 f32 类型的极值计算创建基准测试
    c.bench_function("extremes f32", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f32>(); // 获取挪威主线作为 f32 多边形
        let polygon = Polygon::new(norway, vec![]); // 创建多边形对象

        // 在迭代中测试极值计算
        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).extremes());
        });
    });

    // 为 f64 类型的极值计算创建基准测试
    c.bench_function("extremes f64", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f64>(); // 获取挪威主线作为 f64 多边形
        let polygon = Polygon::new(norway, vec![]); // 创建多边形对象

        // 在迭代中测试极值计算
        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).extremes());
        });
    });
}

// 定义基准组和入口点
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
