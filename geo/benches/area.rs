use criterion::{criterion_group, criterion_main, Criterion};
use geo::Area;
use geo::Polygon;

/// 定义用于基准测试的函数
fn criterion_benchmark(c: &mut Criterion) {
    /// 创建一个基准测试函数，名为“area”
    c.bench_function("area", |bencher| {
        /// 使用挪威的主几何数据创建一个多边形
        let norway = geo_test_fixtures::norway_main::<f32>();
        let polygon = Polygon::new(norway, vec![]);

        /// 在迭代中测试多边形的签名面积计算
        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).signed_area());
        });
    });
}

/// 定义基准组和入口点
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
