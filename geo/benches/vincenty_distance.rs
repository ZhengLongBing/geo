use criterion::{criterion_group, criterion_main};
use geo::algorithm::VincentyDistance;

// 定义基准测试函数
fn criterion_benchmark(c: &mut criterion::Criterion) {
    // 为 f32 类型的文森迪距离进行基准测试
    c.bench_function("vincenty distance f32", |bencher| {
        // 定义两个地理坐标点
        let a = geo::Point::<f32>::new(17.107558, 48.148636);
        let b = geo::Point::<f32>::new(16.372477, 48.20881);

        bencher.iter(|| {
            let _ = criterion::black_box(
                criterion::black_box(&a).vincenty_distance(criterion::black_box(&b)),
            );
        });
    });

    // 为 f64 类型的文森迪距离进行基准测试
    c.bench_function("vincenty distance f64", |bencher| {
        // 定义两个地理坐标点
        let a = geo::Point::new(17.107558, 48.148636);
        let b = geo::Point::new(16.372477, 48.208810);

        bencher.iter(|| {
            let _ = criterion::black_box(
                criterion::black_box(&a).vincenty_distance(criterion::black_box(&b)),
            );
        });
    });
}

// 定义基准测试组和主函数
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
