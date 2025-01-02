use criterion::{criterion_group, criterion_main, Criterion};
use geo::prelude::*;
use geo::{Coord, CoordNum};

use num_traits::Signed;
use rand::distributions::uniform::SampleUniform;
use rand::Rng;

// 生成指定范围内的均匀随机点
pub fn uniform_points_in_range<S: CoordNum + SampleUniform + Signed, R: Rng>(
    range: S,
    size: usize,
    rng: &mut R,
) -> Vec<Coord<S>> {
    (0..size)
        .map(|_| (rng.gen_range(-range..=range), rng.gen_range(-range..=range)).into())
        .collect()
}

// 定义基准测试函数
fn criterion_benchmark(c: &mut Criterion) {
    // 为 f32 类型的凸壳计算创建基准测试
    c.bench_function("convex hull f32", |bencher| {
        let line_string = geo_test_fixtures::norway_main::<f32>();

        // 在迭代中测试凸壳计算
        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&line_string).convex_hull());
        });
    });

    // 为 f64 类型的凸壳计算创建基准测试
    c.bench_function("convex hull f64", |bencher| {
        let line_string = geo_test_fixtures::norway_main::<f64>();

        // 在迭代中测试凸壳计算
        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&line_string).convex_hull());
        });
    });

    // 为包含共线随机 i64 的凸壳计算创建基准测试
    c.bench_function("convex hull with collinear random i64", |bencher| {
        let mut points = uniform_points_in_range(10_000_i64, 1_000_000, &mut rand::thread_rng());
        use geo::convex_hull::graham_hull;
        // 在迭代中测试共线凸壳计算
        bencher.iter(|| {
            criterion::black_box(graham_hull(
                criterion::black_box(&mut points),
                criterion::black_box(true),
            ));
        });
    });
}

// 定义基准组和入口点
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
