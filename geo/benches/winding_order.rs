use criterion::{criterion_group, criterion_main};
use geo::algorithm::Winding; // 导入Winding算法

fn criterion_benchmark(c: &mut criterion::Criterion) {
    // 基准测试：winding_order方法，精度为f32
    c.bench_function("winding order: winding_order (f32)", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();

        bencher.iter(|| {
            let _ = criterion::black_box(criterion::black_box(&ls).winding_order());
        });
    });

    // 基准测试：winding_order方法，精度为f64
    c.bench_function("winding order: winding_order (f64)", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f64>();

        bencher.iter(|| {
            let _ = criterion::black_box(criterion::black_box(&ls).winding_order());
        });
    });

    // 基准测试：points_cw方法，精度为f32
    c.bench_function("winding order: points_cw (f32)", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();

        bencher.iter(|| {
            let points_iter = criterion::black_box(criterion::black_box(&ls).points_cw());
            for point in points_iter {
                criterion::black_box(point);
            }
        });
    });

    // 基准测试：points_cw方法，精度为f64
    c.bench_function("winding order: points_cw (f64)", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f64>();

        bencher.iter(|| {
            let points_iter = criterion::black_box(criterion::black_box(&ls).points_cw());
            for point in points_iter {
                criterion::black_box(point);
            }
        });
    });

    // 基准测试：points_ccw方法，精度为f32
    c.bench_function("winding order: points_ccw (f32)", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();

        bencher.iter(|| {
            let points_iter = criterion::black_box(criterion::black_box(&ls).points_ccw());
            for point in points_iter {
                criterion::black_box(point);
            }
        });
    });

    // 基准测试：points_ccw方法，精度为f64
    c.bench_function("winding order: points_ccw (f64)", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f64>();

        bencher.iter(|| {
            let points_iter = criterion::black_box(criterion::black_box(&ls).points_ccw());
            for point in points_iter {
                criterion::black_box(point);
            }
        });
    });
}

// 定义基准测试组和主函数
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
