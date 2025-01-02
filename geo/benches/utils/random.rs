#![allow(unused)]
use std::f64::consts::PI;

use geo::algorithm::{BoundingRect, ConcaveHull, ConvexHull, MapCoords, Rotate};
use geo::geometry::*;

use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal, Standard};

// TODO: @rmanoka 想知道：是否可以直接将这些随机几何采样器作为 `Distribution::sample` 的实现提供。但这需要在 geo-types 中实现。这可能与我们关于处理 geo-types 中的 private-utils 的讨论有关，但这也可以合并到一个功能标志中。

#[inline]
// 生成在指定矩形范围内的均匀随机点
pub fn uniform_point<R: Rng>(rng: &mut R, bounds: Rect<f64>) -> Coord<f64> {
    let coords: [f64; 2] = rng.sample(Standard);
    let dims = bounds.max() - bounds.min();
    Coord {
        x: bounds.min().x + dims.x * coords[0],
        y: bounds.min().y + dims.y * coords[1],
    }
}

#[inline]
// 生成在指定矩形范围内的均匀随机线段
pub fn uniform_line<R: Rng>(rng: &mut R, bounds: Rect<f64>) -> Line<f64> {
    Line::new(uniform_point(rng, bounds), uniform_point(rng, bounds))
}

#[inline]
// 生成具有指定长度并在指定矩形范围内的均匀随机线段
pub fn uniform_line_with_length<R: Rng>(rng: &mut R, bounds: Rect<f64>, length: f64) -> Line<f64> {
    let start = uniform_point(rng, bounds);
    let line = Line::new(start, start + (length, 0.).into());
    let angle = rng.sample::<f64, _>(Standard) * 2. * PI;
    line.rotate_around_point(angle, start.into())
}

// 定义一个生成器函数，返回一个闭包用于生成扩展的随机线段
pub fn scaled_generator(dims: Coord<f64>, scale: usize) -> impl Fn() -> Line<f64> {
    let scaling: f64 = (1 << scale) as f64;
    let bounds = Rect::new([0., 0.].into(), dims / scaling);
    let shift_bounds = Rect::new([0., 0.].into(), dims - (dims / scaling));

    move || {
        let shift = uniform_point(&mut thread_rng(), shift_bounds);
        uniform_line(&mut thread_rng(), bounds).map_coords(|mut c| {
            c.x += shift.x;
            c.y += shift.y;
            c
        })
    }
}

// 生成一个由圆形分布的点构成的多边形
pub fn circular_polygon<R: Rng>(mut rng: R, steps: usize) -> Polygon<f64> {
    let mut ring = Vec::with_capacity(steps);
    let ang_step = 2. * PI / steps as f64;
    // let ang_nudge = ang_step / 100.;

    let sn = Normal::<f64>::new(0.0, 1.0).unwrap();
    let mut angle: f64 = 0.0;
    (0..steps).for_each(|_| {
        let r: f64 = sn.sample(&mut rng).abs() + 0.1;

        // let ang_nudge = sn.sample(&mut rng) * ang_nudge;
        // angle += ang_nudge;

        let (sin, cos) = angle.sin_cos();
        ring.push((r * cos, r * sin).into());

        angle += ang_step;
    });

    Polygon::new(LineString(ring), vec![])
}

// 生成一个阶梯状的多边形
pub fn steppy_polygon<R: Rng>(mut rng: R, steps: usize) -> Polygon<f64> {
    let mut ring = Vec::with_capacity(2 * steps);

    let y_step = 10.0;
    let nudge_std = y_step / 1000.0;
    let mut y = 0.0;
    let normal = Normal::new(0.0, nudge_std * nudge_std).unwrap();
    let x_shift = 100.0;

    ring.push((0.0, 0.0).into());
    (0..steps).for_each(|_| {
        let x: f64 = rng.sample::<f64, _>(Standard);
        y += y_step;
        ring.push((x, y).into());
    });
    ring.push((x_shift, y).into());
    (0..steps).for_each(|_| {
        let x: f64 = rng.sample::<f64, _>(Standard);
        y -= y_step;
        // y += normal.sample(&mut rng);
        ring.push((x_shift + x, y).into());
    });

    normalize_polygon(Polygon::new(LineString(ring), vec![]))
}

/// 将多边形归一化以适应和填充 `[-1, 1] X [-1, 1]` 方形。
///
/// 使用 `MapCoord` 和 `BoundingRect`
pub fn normalize_polygon(poly: Polygon<f64>) -> Polygon<f64> {
    let bounds = poly.bounding_rect().unwrap();
    let dims = bounds.max() - bounds.min();
    let x_scale = 2. / dims.x;
    let y_scale = 2. / dims.y;

    let x_shift = -bounds.min().x * x_scale - 1.;
    let y_shift = -bounds.min().y * y_scale - 1.;
    poly.map_coords(|mut c| {
        c.x *= x_scale;
        c.x += x_shift;
        c.y *= y_scale;
        c.y += y_shift;
        c
    })
}

// 定义一个包裹样本的结构体
#[derive(Debug, Clone)]
pub struct Samples<T>(Vec<T>);

impl<T> Samples<T> {
    // 返回一个采样器函数，依次获取样本中的元素
    pub fn sampler<'a>(&'a self) -> impl FnMut() -> &'a T {
        let mut curr = 0;
        move || {
            let ret = curr;
            curr += 1;
            curr %= self.0.len();
            &self.0[ret]
        }
    }

    // 从一个生成函数创建样本集合
    pub fn from_fn<F: FnMut() -> T>(size: usize, mut proc: F) -> Self {
        Self((0..size).map(|_| proc()).collect())
    }

    // 映射转换样本中的每一个元素
    pub fn map<U, F: FnMut(T) -> U>(self, mut proc: F) -> Samples<U> {
        Samples(self.0.into_iter().map(proc).collect())
    }
}
