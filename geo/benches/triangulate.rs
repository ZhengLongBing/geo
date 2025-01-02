use criterion::{criterion_group, criterion_main};
use geo::algorithm::{TriangulateEarcut, TriangulateSpade};
use geo::geometry::Polygon;
use geo::triangulate_spade::SpadeTriangulationConfig;

// 定义基准测试函数
fn criterion_benchmark(c: &mut criterion::Criterion) {
    // 基准测试：TriangulateSpade（无约束）- 小型多边形
    c.bench_function(
        "TriangulateSpade (unconstrained) - small polys",
        |bencher| {
            let multi_poly = geo_test_fixtures::nl_zones::<f64>();
            bencher.iter(|| {
                for poly in &multi_poly {
                    let triangulation =
                        TriangulateSpade::unconstrained_triangulation(poly).unwrap(); // 执行无约束三角剖分
                    assert!(triangulation.len() > 1); // 确保结果至少有一个三角形
                    criterion::black_box(triangulation); // 防止优化移除代码
                }
            });
        },
    );

    // 基准测试：TriangulateSpade（有约束）- 小型多边形
    c.bench_function("TriangulateSpade (constrained) - small polys", |bencher| {
        let multi_poly = geo_test_fixtures::nl_zones::<f64>();
        bencher.iter(|| {
            for poly in &multi_poly {
                let triangulation = TriangulateSpade::constrained_triangulation(
                    poly,
                    SpadeTriangulationConfig { snap_radius: 1e-8 }, // 设置约束参数
                )
                .unwrap(); // 执行有约束三角剖分
                assert!(triangulation.len() > 1); // 确保结果至少有一个三角形
                criterion::black_box(triangulation); // 防止优化移除代码
            }
        });
    });

    // 基准测试：TriangulateEarcut - 小型多边形
    c.bench_function("TriangulateEarcut - small polys", |bencher| {
        let multi_poly = geo_test_fixtures::nl_zones::<f64>();
        bencher.iter(|| {
            for poly in &multi_poly {
                let triangulation = TriangulateEarcut::earcut_triangles(poly); // 执行耳切三角剖分
                assert!(triangulation.len() > 1); // 确保结果至少有一个三角形
                criterion::black_box(triangulation); // 防止优化移除代码
            }
        });
    });

    // 基准测试：TriangulateSpade（无约束）- 大型多边形
    c.bench_function("TriangulateSpade (unconstrained) - large_poly", |bencher| {
        let poly = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
        bencher.iter(|| {
            let triangulation = TriangulateSpade::unconstrained_triangulation(&poly).unwrap(); // 执行无约束三角剖分
            assert!(triangulation.len() > 1); // 确保结果至少有一个三角形
            criterion::black_box(triangulation); // 防止优化移除代码
        });
    });

    // 基准测试：TriangulateSpade（有约束）- 大型多边形
    c.bench_function("TriangulateSpade (constrained) - large_poly", |bencher| {
        let poly = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
        bencher.iter(|| {
            let triangulation = TriangulateSpade::constrained_triangulation(
                &poly,
                SpadeTriangulationConfig { snap_radius: 1e-8 }, // 设置约束参数
            )
            .unwrap(); // 执行有约束三角剖分
            assert!(triangulation.len() > 1); // 确保结果至少有一个三角形
            criterion::black_box(triangulation); // 防止优化移除代码
        });
    });

    // 基准测试：TriangulateEarcut - 大型多边形
    c.bench_function("TriangulateEarcut - large_poly", |bencher| {
        let poly = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
        bencher.iter(|| {
            let triangulation = TriangulateEarcut::earcut_triangles(&poly); // 执行耳切三角剖分
            assert!(triangulation.len() > 1); // 确保结果至少有一个三角形
            criterion::black_box(triangulation); // 防止优化移除代码
        });
    });
}

// 定义基准测试组和主函数
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
