use criterion::{criterion_group, criterion_main, Criterion};
use geo::intersects::Intersects;
use geo::MultiPolygon;

// 定义多多边形交错的基准测试函数
fn multi_polygon_intersection(c: &mut Criterion) {
    let plot_polygons: MultiPolygon = geo_test_fixtures::nl_plots_wgs84(); // 获取地块的多多边形
    let zone_polygons: MultiPolygon = geo_test_fixtures::nl_zones(); // 获取区域的多多边形

    c.bench_function("MultiPolygon intersects", |bencher| {
        bencher.iter(|| {
            let mut intersects = 0; // 计数交错的多边形
            let mut non_intersects = 0; // 计数未交错的多边形

            for a in &plot_polygons {
                for b in &zone_polygons {
                    if criterion::black_box(a.intersects(b)) {
                        intersects += 1;
                    } else {
                        non_intersects += 1;
                    }
                }
            }

            assert_eq!(intersects, 974); // 验证交错计数
            assert_eq!(non_intersects, 27782); // 验证未交错计数
        });
    });
}

// 定义矩形交错的基准测试函数
fn rect_intersection(c: &mut Criterion) {
    use geo::algorithm::BoundingRect;
    use geo::geometry::Rect;
    let plot_bbox: Vec<Rect> = geo_test_fixtures::nl_plots_wgs84()
        .iter()
        .map(|plot| plot.bounding_rect().unwrap()) // 获取地块的边界框
        .collect();
    let zone_bbox: Vec<Rect> = geo_test_fixtures::nl_zones()
        .iter()
        .map(|plot| plot.bounding_rect().unwrap()) // 获取区域的边界框
        .collect();

    c.bench_function("Rect intersects", |bencher| {
        bencher.iter(|| {
            let mut intersects = 0; // 计数交错的矩形
            let mut non_intersects = 0; // 计数未交错的矩形

            for a in &plot_bbox {
                for b in &zone_bbox {
                    if criterion::black_box(a.intersects(b)) {
                        intersects += 1;
                    } else {
                        non_intersects += 1;
                    }
                }
            }

            assert_eq!(intersects, 3054); // 验证交错计数
            assert_eq!(non_intersects, 25702); // 验证未交错计数
        });
    });
}

// 定义点与矩形交错的基准测试函数
fn point_rect_intersection(c: &mut Criterion) {
    use geo::algorithm::{BoundingRect, Centroid};
    use geo::geometry::{Point, Rect};
    let plot_centroids: Vec<Point> = geo_test_fixtures::nl_plots_wgs84()
        .iter()
        .map(|plot| plot.centroid().unwrap()) // 获取地块的质心
        .collect();
    let zone_bbox: Vec<Rect> = geo_test_fixtures::nl_zones()
        .iter()
        .map(|plot| plot.bounding_rect().unwrap()) // 获取区域的边界框
        .collect();

    c.bench_function("Point intersects rect", |bencher| {
        bencher.iter(|| {
            let mut intersects = 0; // 计数交错的点
            let mut non_intersects = 0; // 计数未交错的点

            for a in &plot_centroids {
                for b in &zone_bbox {
                    if criterion::black_box(a.intersects(b)) {
                        intersects += 1;
                    } else {
                        non_intersects += 1;
                    }
                }
            }

            assert_eq!(intersects, 2246); // 验证交错计数
            assert_eq!(non_intersects, 26510); // 验证未交错计数
        });
    });
}

// 定义点与三角形交错的基准测试函数
fn point_triangle_intersection(c: &mut Criterion) {
    use geo::{Centroid, TriangulateEarcut};
    use geo_types::{Point, Triangle};
    let plot_centroids: Vec<Point> = geo_test_fixtures::nl_plots_wgs84()
        .iter()
        .map(|plot| plot.centroid().unwrap()) // 获取地块的质心
        .collect();
    let zone_triangles: Vec<Triangle> = geo_test_fixtures::nl_zones()
        .iter()
        .flat_map(|plot| plot.earcut_triangles_iter()) // 获取区域的三角形
        .collect();

    c.bench_function("Point intersects triangle", |bencher| {
        bencher.iter(|| {
            let mut intersects = 0; // 计数交错的点与三角形
            let mut non_intersects = 0; // 计数未交错的点与三角形

            for a in &plot_centroids {
                for b in &zone_triangles {
                    if criterion::black_box(a.intersects(b)) {
                        intersects += 1;
                    } else {
                        non_intersects += 1;
                    }
                }
            }

            assert_eq!(intersects, 533); // 验证交错计数
            assert_eq!(non_intersects, 5450151); // 验证未交错计数
        });
    });

    c.bench_function("Triangle intersects point", |bencher| {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (5., 10.)]); // 定义三角形
        let point = Point::new(5., 5.); // 定义点

        bencher.iter(|| {
            assert!(criterion::black_box(&triangle).intersects(criterion::black_box(&point)));
            // 验证三角形与点交错
        });
    });

    c.bench_function("Triangle intersects point on edge", |bencher| {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (6., 10.)]); // 定义三角形
        let point = Point::new(3., 5.); // 定义边缘上的点

        bencher.iter(|| {
            assert!(criterion::black_box(&triangle).intersects(criterion::black_box(&point)));
            // 验证三角形边缘与点交错
        });
    });
}

// 定义基准组和入口
criterion_group! {
    name = bench_multi_polygons;
    config = Criterion::default().sample_size(10);
    targets = multi_polygon_intersection
}
criterion_group!(bench_rects, rect_intersection);
criterion_group! {
    name = bench_point_rect;
    config = Criterion::default().sample_size(50);
    targets = point_rect_intersection
}
criterion_group! {
    name = bench_point_triangle;
    config = Criterion::default().sample_size(50);
    targets = point_triangle_intersection
}

// 定义主入口
criterion_main!(
    bench_multi_polygons,
    bench_rects,
    bench_point_rect,
    bench_point_triangle
);
