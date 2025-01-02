use criterion::{criterion_group, criterion_main, Criterion};
use geo::algorithm::Relate;
use geo::PreparedGeometry;
use geo_types::MultiPolygon;

// 定义基准测试函数
fn criterion_benchmark(c: &mut Criterion) {
    // 准备好的多边形之间的关系基准测试
    c.bench_function("relate prepared polygons", |bencher| {
        let plot_polygons: MultiPolygon = geo_test_fixtures::nl_plots_wgs84(); // 获取地块的多边形
        let zone_polygons = geo_test_fixtures::nl_zones(); // 获取区域的多边形

        // 迭代进行基准测试
        bencher.iter(|| {
            let mut intersects = 0; // 计数相交的多边形
            let mut non_intersects = 0; // 计数不相交的多边形

            // 为地块多边形创建准备好的几何体
            let plot_polygons = plot_polygons
                .iter()
                .map(PreparedGeometry::from)
                .collect::<Vec<_>>();

            // 为区域多边形创建准备好的几何体
            let zone_polygons = zone_polygons
                .iter()
                .map(PreparedGeometry::from)
                .collect::<Vec<_>>();

            // 检查每一对多边形之间的关系
            for a in &plot_polygons {
                for b in &zone_polygons {
                    if criterion::black_box(a.relate(b).is_intersects()) {
                        intersects += 1;
                    } else {
                        non_intersects += 1;
                    }
                }
            }

            // 验证相交和不相交的计数
            assert_eq!(intersects, 974);
            assert_eq!(non_intersects, 27782);
        });
    });

    // 未准备好的多边形之间的关系基准测试
    c.bench_function("relate unprepared polygons", |bencher| {
        let plot_polygons: MultiPolygon = geo_test_fixtures::nl_plots_wgs84(); // 获取地块的多边形
        let zone_polygons = geo_test_fixtures::nl_zones(); // 获取区域的多边形

        // 迭代进行基准测试
        bencher.iter(|| {
            let mut intersects = 0; // 计数相交的多边形
            let mut non_intersects = 0; // 计数不相交的多边形

            // 检查每一对多边形之间的关系
            for a in &plot_polygons {
                for b in &zone_polygons {
                    if criterion::black_box(a.relate(b).is_intersects()) {
                        intersects += 1;
                    } else {
                        non_intersects += 1;
                    }
                }
            }

            // 验证相交和不相交的计数
            assert_eq!(intersects, 974);
            assert_eq!(non_intersects, 27782);
        });
    });
}

// 定义基准测试组和主函数
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
