// 引入所需的库和模块
use std::f64::consts::PI; // 使用标准库的PI常量

use criterion::{measurement::Measurement, *}; // 导入criterion库中的测量和所有其他项
use geo::algorithm::{BooleanOps, Rotate}; // 导入geo库中的布尔操作和旋转算法

use rand::{thread_rng, Rng}; // 导入随机数生成库
use rand_distr::Standard; // 导入标准概率分布

// 引入自定义模块random，路径为相对路径
#[path = "../../geo/benches/utils/random.rs"]
mod random;
use random::Samples; // 使用random模块中的Samples函数

// 引入自定义模块bops，路径为相对路径
#[path = "utils/bops.rs"]
mod bops;
use bops::convert_poly; // 使用bops模块中的convert_poly函数

// 定义基准测试函数run_complex
fn run_complex<T: Measurement>(c: &mut Criterion<T>) {
    const SAMPLE_SIZE: usize = 16; // 定义样本大小常量
    let mut group = c.benchmark_group("Circular polygon boolean-ops"); // 创建新的基准测试组
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic)); // 设置图表配置

    // 循环遍历每个尺度
    (8..14).for_each(|scale| {
        let steps = 1 << scale; // 计算步数
        let polys = Samples::from_fn(SAMPLE_SIZE, || {
            // 生成有阶多边形
            let poly1 = random::steppy_polygon(thread_rng(), steps);
            // 随机角度旋转该多边形
            let angle: f64 = thread_rng().sample::<f64, _>(Standard) * PI * 2.0;
            let poly1 = poly1.rotate_around_point(angle, poly1.exterior().0[0].into());

            // 生成圆形多边形
            let poly2 = random::circular_polygon(thread_rng(), steps);
            // 随机角度旋转该多边形
            let angle: f64 = thread_rng().sample::<f64, _>(Standard) * PI * 2.0;
            let poly2 = poly2.rotate_around_point(angle, poly2.exterior().0[0].into());

            // 转换多边形格式
            let cp1 = convert_poly(&poly1);
            let cp2 = convert_poly(&poly2);
            (poly1, poly2, cp1, cp2) // 返回四个多边形
        });

        group.sample_size(10); // 设置样本大小
                               // 为布尔操作相交创建基准测试
        group.bench_with_input(
            BenchmarkId::new("bops::intersection", steps),
            &(),
            |b, _| {
                b.iter_batched(
                    polys.sampler(),
                    |(poly, poly2, _, _)| poly.intersection(poly2),
                    BatchSize::SmallInput,
                );
            },
        );

        // 为布尔操作合并创建基准测试
        group.bench_with_input(BenchmarkId::new("bops::union", steps), &(), |b, _| {
            b.iter_batched(
                polys.sampler(),
                |(poly, poly2, _, _)| poly.union(poly2),
                BatchSize::SmallInput,
            );
        });

        // 如果启用了bench-foreign-booleanop特性，执行以下基准测试
        #[cfg(feature = "bench-foreign-booleanop")]
        {
            use geo::algorithm::Relate;
            use geo_booleanop::boolean::BooleanOp as OtherBooleanOp;

            // 为其他布尔操作相交创建基准测试
            group.bench_with_input(
                BenchmarkId::new("rgbops::intersection", steps),
                &(),
                |b, _| {
                    b.iter_batched(
                        polys.sampler(),
                        |(_, _, poly, poly2)| OtherBooleanOp::intersection(poly, poly2),
                        BatchSize::SmallInput,
                    );
                },
            );

            // 为其他布尔操作合并创建基准测试
            group.bench_with_input(BenchmarkId::new("rgbops::union", steps), &(), |b, _| {
                b.iter_batched(
                    polys.sampler(),
                    |(_, _, poly, poly2)| OtherBooleanOp::union(poly, poly2),
                    BatchSize::SmallInput,
                );
            });

            // 为地理关系创建基准测试
            group.bench_with_input(BenchmarkId::new("geo::relate", steps), &(), |b, _| {
                b.iter_batched(
                    polys.sampler(),
                    |(poly, poly2, _, _)| poly.relate(poly2).is_intersects(),
                    BatchSize::SmallInput,
                );
            });
        }
    });
}

// 定义基准测试组和主函数
criterion_group!(verts_vs_time, run_complex);
criterion_main!(verts_vs_time);
