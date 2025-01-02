use criterion::{criterion_group, criterion_main, Criterion};
use geo::algorithm::{Contains, Convert, Relate};
use geo::geometry::*;
use geo::{coord, point, polygon};

// 定义基准测试函数
fn criterion_benchmark(c: &mut Criterion) {
    // 测试点在简单多边形内部
    c.bench_function("point in simple polygon", |bencher| {
        let polygon = polygon![
            (x: 0.0f64, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ];
        let point = Point::new(0.5, 0.1); // 创建一个点
        bencher.iter(|| {
            // 验证点在多边形内
            assert!(criterion::black_box(&polygon).contains(criterion::black_box(&point)));
        });
    });

    // 测试点在简单多边形外部
    c.bench_function("point outside simple polygon", |bencher| {
        let polygon = polygon![
            (x: 0.0f64, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ];
        let point = Point::new(2.0, 2.0); // 创建一个点位于多边形外部
        bencher.iter(|| {
            // 验证点不在多边形内
            assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&point)));
        });
    });

    // 测试点在复杂多边形内
    c.bench_function("point inside complex polygon", |bencher| {
        let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]); // 创建复杂多边形
        let point = geo_test_fixtures::baton_rouge(); // 创建点
        bencher.iter(|| {
            // 验证点在多边形内
            assert!(criterion::black_box(&polygon).contains(criterion::black_box(&point)));
        });
    });

    // 测试点在复杂多边形的包围盒内但在多边形外部
    c.bench_function(
        "point outside, but within bbox, of complex polygon",
        |bencher| {
            let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
            // lake borgne - 接近Louisiana但在其外部
            let point = point!(x: -89.641854, y: 30.026283);
            bencher.iter(|| {
                // 验证点不在多边形内
                assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&point)));
            });
        },
    );

    // 测试点在复杂多边形的包围盒外部
    c.bench_function("point outside bbox of complex polygon", |bencher| {
        let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
        let point = point!(x: 2.3522, y: 48.8566);
        bencher.iter(|| {
            // 验证点不在多边形内
            assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&point)));
        });
    });

    // 测试点水平排列到梳形边缘
    c.bench_function(
        "point horizontal to comb teeth aka bart's haircut",
        |bencher| {
            // 测试一个病态案例，其中点水平位于许多边缘。
            //
            // 梳形结构 -> |\\/\\/\\/\\/|      *  <---点
            //               |____________|
            let polygon = polygon!(
                (x: 0 ,y: 0),
                (x: 0 ,y: 10),
                (x: 1 ,y: 5),
                (x: 2 ,y: 10),
                (x: 3 ,y: 5),
                (x: 4 ,y: 10),
                (x: 5 ,y: 5),
                (x: 6 ,y: 10),
                (x: 7 ,y: 5),
                (x: 8 ,y: 10),
                (x: 9 ,y: 10),
                (x: 10,y:  10),
                (x: 10,y:  0),
                (x: 0 ,y: 0)
            );
            let point = point!(x: 20, y: 7);

            bencher.iter(|| {
                // 验证点不在多边形内
                assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&point)));
            })
        },
    );

    // 测试线穿过复杂多边形
    c.bench_function("line across complex polygon", |bencher| {
        let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
        // 穿过但不被Louisiana包含
        let line = Line::new(
            geo_test_fixtures::baton_rouge(),
            point!(x: -89.641854, y: 30.026283),
        );
        bencher.iter(|| {
            // 验证线不在多边形内
            assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&line)));
        });
    });

    // 测试复杂多边形包含另一个多边形
    c.bench_function("complex polygon contains polygon", |bencher| {
        let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
        let contained_polygon = geo_test_fixtures::east_baton_rouge();

        bencher.iter(|| {
            // 验证多边形内包含另一个多边形
            assert!(
                criterion::black_box(&polygon).contains(criterion::black_box(&contained_polygon))
            );
        });
    });

    // 测试三角形包含点
    c.bench_function("Triangle contains point", |bencher| {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (5., 10.)]); // 创建三角形
        let point = Point::new(5., 5.); // 创建点

        bencher.iter(|| {
            // 验证三角形内包含点
            assert!(criterion::black_box(&triangle).contains(criterion::black_box(&point)));
        });
    });

    // 测试三角形包含点在边上
    c.bench_function("Triangle contains point on edge", |bencher| {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (6., 10.)]); // 创建三角形
        let point = Point::new(3., 5.); // 创建点在边上

        bencher.iter(|| {
            // 验证点不在三角形内
            assert!(!criterion::black_box(&triangle).contains(criterion::black_box(&point)));
        });
    });

    // 测试矩形包含多边形
    c.bench_function("Rect contains polygon", |bencher| {
        let polygon = polygon![
            (x: 150., y: 350.),
            (x: 100., y: 350.),
            (x: 210., y: 160.),
            (x: 290., y: 350.),
            (x: 250., y: 350.),
            (x: 200., y: 250.),
            (x: 150., y: 350.),
        ];
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. }); // 创建矩形

        bencher.iter(|| {
            // 验证矩形内包含多边形
            assert!(criterion::black_box(&rect).contains(criterion::black_box(&polygon)));
        });
    });

    // 测试LineString不包含另一个LineString（Contains trait）
    c.bench_function(
        "LineString not contains LineString (Contains trait)",
        |bencher| {
            let ls_1: geo::LineString<f64> = geo_test_fixtures::poly1(); // 创建第一个线串
            let ls_2: geo::LineString<f64> = geo_test_fixtures::poly2(); // 创建第二个线串

            bencher.iter(|| {
                // 验证第一个线串不包含第二个线串
                assert!(!ls_1.contains(&ls_2));
            });
        },
    );

    // 测试LineString不包含另一个LineString（Relate trait）
    c.bench_function(
        "LineString not contains LineString (Relate trait)",
        |bencher| {
            let ls_1: geo::LineString<f64> = geo_test_fixtures::poly1(); // 创建第一个线串
            let ls_2: geo::LineString<f64> = geo_test_fixtures::poly2(); // 创建第二个线串

            bencher.iter(|| {
                // 验证第一个线串不关联包含第二个线串
                assert!(!ls_1.relate(&ls_2).is_contains());
            });
        },
    );

    // 测试LineString包含另一个LineString（Contains trait）
    c.bench_function(
        "LineString contains LineString (Contains trait)",
        |bencher| {
            let ls_1: LineString<f64> = geo_test_fixtures::poly1(); // 创建第一个线串
            let mut ls_2 = LineString::new(ls_1.0[1..].to_vec()); // 创建第二个线串是第一个的子集
            ls_2.0.pop(); // 去掉最后一个点

            bencher.iter(|| {
                // 验证第一个线串包含第二个线串
                assert!(ls_1.contains(&ls_2));
            });
        },
    );

    // 测试LineString包含另一个LineString（Relate trait）
    c.bench_function("LineString contains LineString (Relate trait)", |bencher| {
        let ls_1: geo::LineString<f64> = geo_test_fixtures::poly1(); // 创建第一个线串
        let mut ls_2 = LineString::new(ls_1.0[1..].to_vec()); // 创建第二个线串是第一个的子集
        ls_2.0.pop(); // 去掉最后一个点

        bencher.iter(|| {
            // 验证第一个线串关联包含第二个线串
            assert!(ls_1.relate(&ls_2).is_contains());
        });
    });

    // 测试MultiPolygon包含MultiPoint（Contains trait）
    c.bench_function("MultiPolygon contains MultiPoint (Contains trait)", |bencher| {
        let p_1: Polygon<f64> = Polygon::new(geo_test_fixtures::poly1(), vec![]); // 创建第一个多边形
        let p_2: Polygon<f64> = Polygon::new(geo_test_fixtures::poly2(), vec![]); // 创建第二个多边形
        let multi_poly = MultiPolygon(vec![p_1, p_2]); // 创建多多边形
        let multi_point: MultiPoint<f64> = geo::wkt!(MULTIPOINT (-60 10,-60 -70,-120 -70,-120 10,-40 80,30 80,30 10,-40 10,100 210,100 120,30 120,30 210,-185 -135,-100 -135,-100 -230,-185 -230)).convert();

        bencher.iter(|| {
            // 验证多多边形包含多个点
            assert!(multi_poly.contains(&multi_point));
        });
    });

    // 测试MultiPolygon包含MultiPoint（Relate trait）
    c.bench_function("MultiPolygon contains MultiPoint (Relate trait)", |bencher| {
        let p_1: Polygon<f64> = Polygon::new(geo_test_fixtures::poly1(), vec![]); // 创建第一个多边形
        let p_2: Polygon<f64> = Polygon::new(geo_test_fixtures::poly2(), vec![]); // 创建第二个多边形
        let multi_poly = MultiPolygon(vec![p_1, p_2]); // 创建多多边形
        let multi_point: MultiPoint<f64> = geo::wkt!(MULTIPOINT (-60 10,-60 -70,-120 -70,-120 10,-40 80,30 80,30 10,-40 10,100 210,100 120,30 120,30 210,-185 -135,-100 -135,-100 -230,-185 -230)).convert();

        bencher.iter(|| {
            // 验证多多边形关联包含多个点
            assert!(multi_poly.relate(&multi_point).is_contains());
        });
    });

    // 测试MultiPolygon不包含MultiPoint（Contains trait）
    c.bench_function("MultiPolygon not contains MultiPoint (Contains trait)", |bencher| {
        let p_1: Polygon<f64> = Polygon::new(geo_test_fixtures::poly1(), vec![]); // 创建第一个多边形
        let p_2: Polygon<f64> = Polygon::new(geo_test_fixtures::poly2(), vec![]); // 创建第二个多边形
        let multi_poly = MultiPolygon(vec![p_1, p_2]); // 创建多多边形
        let multi_point: MultiPoint<f64> = geo::wkt!(MULTIPOINT (-160 10,-60 -70,-120 -70,-120 10,-40 80,30 80,30 10,-40 10,100 210,100 120,30 120,30 210,-185 -135,-100 -135,-100 -230,-185 -230)).convert();

        bencher.iter(|| {
            // 验证多多边形不包含多个点
            assert!(multi_poly.contains(&multi_point));
        });
    });

    // 测试MultiPolygon不包含MultiPoint（Relate trait）
    c.bench_function("MultiPolygon not contains MultiPoint (Relate trait)", |bencher| {
        let p_1: Polygon<f64> = Polygon::new(geo_test_fixtures::poly1(), vec![]); // 创建第一个多边形
        let p_2: Polygon<f64> = Polygon::new(geo_test_fixtures::poly2(), vec![]); // 创建第二个多边形
        let multi_poly = MultiPolygon(vec![p_1, p_2]); // 创建多多边形
        let multi_point: MultiPoint<f64> = geo::wkt!(MULTIPOINT (-160 10,-60 -70,-120 -70,-120 10,-40 80,30 80,30 10,-40 10,100 210,100 120,30 120,30 210,-185 -135,-100 -135,-100 -230,-185 -230)).convert();

        bencher.iter(|| {
            // 验证多多边形不关联包含多个点
            assert!(multi_poly.relate(&multi_point).is_contains());
        });
    });
}

// 定义基准组和入口点
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
