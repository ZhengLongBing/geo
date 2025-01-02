use std::fmt::Display;
use std::panic::catch_unwind;

use criterion::measurement::Measurement;
use geo::coordinate_position::CoordPos;
use geo::monotone::monotone_subdivision;
use geo::{CoordinatePosition, MapCoords, Polygon};

use criterion::{
    criterion_group, criterion_main, BatchSize, BenchmarkGroup, BenchmarkId, Criterion,
};
use geo_types::{Coord, Rect};
use wkt::ToWkt;

#[path = "utils/random.rs"]
mod random;
use rand::thread_rng;
use random::*;

// 定义点在多边形中的基准测试
fn criterion_benchmark_pt_in_poly(c: &mut Criterion) {
    // 创建点样本
    let pt_samples = Samples::from_fn(512, || {
        uniform_point(&mut thread_rng(), Rect::new((-1., -1.), (1., 1.)))
    });

    // 测试不同大小的“踏步多边形（最坏情况）”
    for size in [16, 64, 512, 1024, 2048] {
        let mut grp = c.benchmark_group("rand pt-in-poly steppy-polygon (worst case)".to_string());
        let poly = steppy_polygon(&mut thread_rng(), size);
        bench_pt_in_poly(&mut grp, poly, size, &pt_samples)
    }

    // 测试不同大小的“踏步多边形（最好情况）”
    for size in [16, 64, 512, 1024, 2048] {
        let mut grp = c.benchmark_group("rand pt-in-poly steppy-polygon (best case)".to_string());
        let poly = steppy_polygon(&mut thread_rng(), size).map_coords(|c| (c.y, c.x).into());
        bench_pt_in_poly(&mut grp, poly, size, &pt_samples)
    }

    // 测试不同大小的“圆形多边形”
    for size in [16, 64, 512, 1024, 2048] {
        let mut grp = c.benchmark_group("rand pt-in-poly circular-polygon".to_string());
        let poly = circular_polygon(&mut thread_rng(), size);
        bench_pt_in_poly(&mut grp, poly, size, &pt_samples)
    }
}

// 辅助函数：进行点在多边形中的基准测试
fn bench_pt_in_poly<T, I>(
    g: &mut BenchmarkGroup<T>,
    polygon: Polygon<f64>,
    param: I,
    samples: &Samples<Coord<f64>>,
) where
    T: Measurement,
    I: Display + Copy,
{
    // 使用单调分割进行预处理
    let mon = match catch_unwind(|| monotone_subdivision([polygon.clone()])) {
        Ok(m) => m,
        Err(_) => {
            panic!(
                "Monotone subdivision failed for polygon: {}",
                polygon.to_wkt()
            );
        }
    };

    // 基准测试：简单的点在多边形内
    g.bench_with_input(
        BenchmarkId::new("Simple point-in-poly", param),
        &(),
        |b, _| {
            b.iter_batched(
                samples.sampler(),
                |pt| {
                    polygon.coordinate_position(pt);
                },
                BatchSize::SmallInput,
            );
        },
    );

    // 基准测试：预处理后的点在多边形内
    g.bench_with_input(
        BenchmarkId::new("Pre-processed point-in-poly", param),
        &(),
        |b, _| {
            b.iter_batched(
                samples.sampler(),
                |pt| {
                    mon.iter()
                        .filter(|mp| mp.coordinate_position(pt) == CoordPos::Inside)
                        .count();
                },
                BatchSize::SmallInput,
            );
        },
    );
}

// 定义单调分割的基准测试
fn criterion_benchmark_monotone_subdiv(c: &mut Criterion) {
    // 测试不同大小的“踏步多边形（最坏情况）”
    for size in [16, 64, 2048, 32000] {
        let mut grp = c.benchmark_group("monotone_subdiv steppy-polygon (worst case)".to_string());
        let poly_fn = |size| steppy_polygon(&mut thread_rng(), size);
        bench_monotone_subdiv(&mut grp, poly_fn, size)
    }

    // 测试不同大小的“踏步多边形（最好情况）”
    for size in [16, 64, 2048, 32000] {
        let mut grp = c.benchmark_group("monotone_subdiv steppy-polygon (best case)".to_string());
        let poly_fn =
            |size| steppy_polygon(&mut thread_rng(), size).map_coords(|c| (c.y, c.x).into());
        bench_monotone_subdiv(&mut grp, poly_fn, size)
    }

    // 测试不同大小的“圆形多边形”
    for size in [16, 64, 2048, 32000] {
        let mut grp = c.benchmark_group("monotone_subdiv circular-polygon".to_string());
        let poly_fn = |size| circular_polygon(&mut thread_rng(), size);
        bench_monotone_subdiv(&mut grp, poly_fn, size)
    }
}

// 辅助函数：进行单调分割的基准测试
fn bench_monotone_subdiv<T, F>(g: &mut BenchmarkGroup<T>, mut f: F, param: usize)
where
    T: Measurement,
    F: FnMut(usize) -> Polygon<f64>,
{
    // 创建样本集
    let samples = Samples::from_fn(16, || f(param));

    // 基准测试：单调分割
    g.bench_with_input(
        BenchmarkId::new("Montone subdivision", param),
        &(),
        |b, _| {
            b.iter_batched(
                samples.sampler(),
                |pt| {
                    let mon = monotone_subdivision([pt.clone()]);
                    mon.len();
                },
                BatchSize::SmallInput,
            );
        },
    );
}

// 定义基准组和入口点
criterion_group!(
    benches,
    criterion_benchmark_pt_in_poly,
    criterion_benchmark_monotone_subdiv
);
criterion_main!(benches);
