use std::fmt::Display;

use criterion::{measurement::Measurement, *};
use geo::{Coord, Line};

const BBOX: Coord<f64> = Coord { x: 1024., y: 1024. };

#[path = "utils/random.rs"]
mod random;

#[path = "utils/crossings.rs"]
mod crossings;
use crossings::*;
use random::*;

// 基准测试算法函数
fn bench_algos<T, F, I>(g: &mut BenchmarkGroup<T>, mut gen: F, sample_size: usize, param: I)
where
    T: Measurement,
    F: FnMut() -> Vec<Line<f64>>,
    I: Display + Copy,
{
    // 生成样本数据
    let samples = Samples::from_fn(sample_size, || {
        let lines = gen();
        let expected = count_brute(&lines); // 计算预期交叉次数
        (lines, expected)
    });

    // 以输入参数进行基准测试 - Bentley-Ottman算法
    g.bench_with_input(BenchmarkId::new("Bentley-Ottman", param), &(), |b, _| {
        b.iter_batched(
            samples.sampler(),
            |lines| {
                assert_eq!(count_bo(&lines.0), lines.1); // 验证交叉次数是否正确
            },
            BatchSize::SmallInput,
        );
    });

    // 以输入参数进行基准测试 - 暴力算法
    g.bench_with_input(BenchmarkId::new("Brute-Force", param), &(), |b, _| {
        b.iter_batched(
            samples.sampler(),
            |lines| {
                assert_eq!(count_brute(&lines.0), lines.1); // 验证交叉次数是否正确
            },
            BatchSize::SmallInput,
        );
    });

    // 以输入参数进行基准测试 - R-Tree算法
    g.bench_with_input(BenchmarkId::new("R-Tree", param), &(), |b, _| {
        b.iter_batched(
            samples.sampler(),
            |lines| {
                assert_eq!(count_rtree(&lines.0), lines.1); // 验证交叉次数是否正确
            },
            BatchSize::SmallInput,
        );
    });
}

// 短线测试函数
fn short<T: Measurement>(c: &mut Criterion<T>) {
    const NUM_LINES: usize = 4096; // 线段数量
    const SAMPLE_SIZE: usize = 10; // 样本大小

    let mut group = c.benchmark_group("Short lines"); // 创建基准测试组
    group.sample_size(10); // 设置样本大小
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic)); // 配置绘图

    (0..10).for_each(|scale| {
        let line_gen = scaled_generator(BBOX, scale);
        let scaling: f64 = (1 << scale) as f64; // 计算缩放比例

        // 执行基准测试
        bench_algos(
            &mut group,
            || (0..NUM_LINES).map(|_| line_gen()).collect(),
            SAMPLE_SIZE,
            1. / scaling,
        );
    });
}

// 随机线段测试函数
fn uniform<T: Measurement>(c: &mut Criterion<T>) {
    const SAMPLE_SIZE: usize = 16; // 样本大小
    const SCALE: usize = 4; // 缩放比例

    let mut group = c.benchmark_group("Random lines"); // 创建基准测试组
    group.sample_size(2 * SAMPLE_SIZE); // 设置样本大小
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic)); // 配置绘图

    let line_gen = scaled_generator(BBOX, SCALE); // 生成线段

    (3..12).step_by(2).for_each(|log_num_lines| {
        let num_lines = 1 << log_num_lines; // 计算总线段数量
                                            // 执行基准测试
        bench_algos(
            &mut group,
            || (0..num_lines).map(|_| line_gen()).collect(),
            SAMPLE_SIZE,
            num_lines,
        );
    });
}

// 混合线段测试函数
fn mixed<T: Measurement>(c: &mut Criterion<T>) {
    const SAMPLE_SIZE: usize = 16; // 样本大小

    let mut group = c.benchmark_group("Mixed"); // 创建基准测试组
    group.sample_size(2 * SAMPLE_SIZE); // 设置样本大小
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic)); // 配置绘图

    (3..12).step_by(2).for_each(|log_num_lines| {
        let num_lines = 1 << log_num_lines; // 计算总线段数量
                                            // 执行基准测试
        bench_algos(
            &mut group,
            || {
                (0..8)
                    .flat_map(|scale| {
                        let line_gen = scaled_generator(BBOX, scale); // 生成线段
                        (0..num_lines / 8).map(move |_| line_gen())
                    })
                    .collect()
            },
            SAMPLE_SIZE,
            num_lines,
        );
    });
}

criterion_group!(random, uniform, short, mixed); // 定义基准测试组
criterion_main!(random); // 设置基准测试入口
