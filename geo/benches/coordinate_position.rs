#[macro_use]
extern crate criterion;
extern crate geo;

use geo::{
    coordinate_position::CoordPos, BoundingRect, Centroid, CoordinatePosition, Point, Rect,
    Triangle,
};

use criterion::Criterion;

// 定义基准测试函数
fn criterion_benchmark(c: &mut Criterion) {
    // 创建用于测试“点对于矩形的位置”的基准测试
    c.bench_function("Point position to rect", |bencher| {
        // 获取挪威块的质心
        let plot_centroids: Vec<Point> = geo_test_fixtures::nl_plots_wgs84()
            .iter()
            .map(|plot| plot.centroid().unwrap())
            .collect();
        // 获取挪威区域的边界矩形
        let zone_bbox: Vec<Rect> = geo_test_fixtures::nl_zones()
            .iter()
            .map(|plot| plot.bounding_rect().unwrap())
            .collect();
        // 对每个质心和边界框进行迭代测试
        bencher.iter(|| {
            let mut inside = 0;
            let mut outsied = 0;
            let mut boundary = 0;

            for a in &plot_centroids {
                for b in &zone_bbox {
                    match criterion::black_box(b).coordinate_position(criterion::black_box(&a.0)) {
                        CoordPos::OnBoundary => boundary += 1,
                        CoordPos::Inside => inside += 1,
                        CoordPos::Outside => outsied += 1,
                    }
                }
            }

            // 验证质心的统计结果
            assert_eq!(inside, 2246);
            assert_eq!(outsied, 26510);
            assert_eq!(boundary, 0);
        });
    });

    // 创建用于测试“点在三角形内”的基准测试
    c.bench_function("Point in triangle", |bencher| {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (5., 10.)]);
        let point = Point::new(5., 5.);

        bencher.iter(|| {
            // 验证点不在三角形外
            assert!(
                criterion::black_box(&triangle).coordinate_position(criterion::black_box(&point.0))
                    != CoordPos::Outside
            );
        });
    });

    // 创建用于测试“点在三角形边界上”的基准测试
    c.bench_function("Point on triangle boundary", |bencher| {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (6., 10.)]);
        let point = Point::new(3., 5.);

        bencher.iter(|| {
            // 验证点在三角形的边界上
            assert!(
                criterion::black_box(&triangle).coordinate_position(criterion::black_box(&point.0))
                    == CoordPos::OnBoundary
            );
        });
    });
}

// 定义基准组和入口点
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
