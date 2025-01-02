use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use geo::algorithm::{Relate, Rotate, Translate};
use geo::geometry::{LineString, Polygon};

fn criterion_benchmark(c: &mut Criterion) {
    // 基准测试重叠的50点多边形
    c.bench_function("relate overlapping 50-point polygons", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f32>();
        let points = norway.0;

        let sub_polygon = {
            let points = points[0..50].to_vec(); // 选取前50个点创建外部线
            let mut exterior = LineString::<f32>::from(points);
            exterior.close(); // 闭合线形成多边形
            Polygon::new(exterior, vec![])
        };

        let polygon = {
            let points = points[40..90].to_vec(); // 从40到90的点创建外部线
            let mut exterior = LineString::<f32>::from(points);
            exterior.close(); // 闭合线形成多边形
            Polygon::new(exterior, vec![])
        };

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&polygon).relate(criterion::black_box(&sub_polygon)), // 测量两个多边形的关系
            );
        });
    });

    // 整个 jts 测试套件基准测试
    c.bench_function("entire jts test suite", |bencher| {
        let mut relate_tests = jts_test_runner::TestRunner::new();
        relate_tests.prepare_cases().unwrap();

        bencher.iter_batched(
            || relate_tests.clone(),
            |mut test_runner| {
                test_runner.run().unwrap(); // 运行测试
                assert!(test_runner.failures().is_empty()); // 确保没有失败的测试
                assert!(!test_runner.successes().is_empty()); // 确保有成功的测试
            },
            BatchSize::SmallInput,
        );
    });

    // 仅匹配“Relate”的 jts 测试套件基准测试
    c.bench_function("jts test suite matching *Relate*", |bencher| {
        let mut relate_tests =
            jts_test_runner::TestRunner::new().matching_filename_glob("*Relate*");
        relate_tests.prepare_cases().unwrap();

        bencher.iter_batched(
            || relate_tests.clone(),
            |mut test_runner| {
                test_runner.run().unwrap(); // 运行相关的测试
                assert!(test_runner.failures().is_empty());
                assert!(!test_runner.successes().is_empty());
            },
            BatchSize::SmallInput,
        );
    });

    // 不相交多边形的基准测试
    c.bench_function("disjoint polygons", |bencher| {
        let norway = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
        let louisiana = Polygon::new(geo_test_fixtures::louisiana::<f64>(), vec![]);

        bencher.iter(|| {
            criterion::black_box(norway.relate(&louisiana)); // 测量挪威和路易斯安那多边形的关系
        });
    });

    // 大旋转多边形的基准测试
    c.bench_function("large rotated polygons", |bencher| {
        let norway = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
        let rotated_norway = norway.rotate_around_center(20.0); // 绕中心旋转

        bencher.iter(|| {
            criterion::black_box(norway.relate(&rotated_norway)); // 测量旋转后的多边形的关系
        });
    });

    // 偏移多边形的基准测试
    c.bench_function("offset polygons", |bencher| {
        let norway = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
        let translated_norway = norway.translate(10.0, 10.0); // 平移多边形

        bencher.iter(|| {
            criterion::black_box(norway.relate(&translated_norway)); // 测量平移后的多边形的关系
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
