mod mono_poly;
use crate::{Coord, GeoNum, Intersects, MultiPolygon, Polygon};
pub use mono_poly::MonoPoly;

mod segment;
use segment::RcSegment;
pub(crate) use segment::Segment;

mod sweep;
pub(crate) use sweep::SimpleSweep;

mod builder;
pub use builder::monotone_subdivision;

/// 一个由多个（不相交）单调多边形组成的多边形集合。
///
/// 该结构针对点在多边形内的查询进行了优化，通常比`Polygon`上的同等方法快得多。
/// 这是因为单个单调多边形可以在`O(log n)`时间内进行点的相交测试，其中`n`是多边形中的顶点数量。
/// 相比之下，`Polygon`上的当量方法是`O(n)`。
/// 通常，一个多边形可以分割为少量单调多边形，从而提供显著的加速。
///
/// # 示例
///
/// 从`Polygon`或`MultiPolygon`构造一个`MonotonicPolygons`，使用`MonotonicPolygons::from`，
/// 并通过`Intersects<Coord>`特性查询点相交。
///
/// ```rust
/// use geo::prelude::*;
/// use geo::{polygon, coord};
///
/// let polygon = polygon![
///     (x: -2., y: 1.),
///     (x: 1., y: 3.),
///     (x: 4., y: 1.),
///     (x: 1., y: -1.),
///     (x: -2., y: 1.),
/// ];
/// let mp = MonotonicPolygons::from(polygon);
/// assert!(mp.intersects(&coord!(x: -2., y: 1.)));
/// ```
#[derive(Clone, Debug)]
pub struct MonotonicPolygons<T: GeoNum>(Vec<MonoPoly<T>>);

impl<T: GeoNum> MonotonicPolygons<T> {
    /// 获取单调多边形的引用。
    pub fn subdivisions(&self) -> &Vec<MonoPoly<T>> {
        &self.0
    }

    /// 将其简化为内在的单调多边形向量。
    pub fn into_subdivisions(self) -> Vec<MonoPoly<T>> {
        self.0
    }
}
impl<T: GeoNum> From<Polygon<T>> for MonotonicPolygons<T> {
    fn from(poly: Polygon<T>) -> Self {
        Self(monotone_subdivision([poly]))
    }
}

impl<T: GeoNum> From<MultiPolygon<T>> for MonotonicPolygons<T> {
    fn from(mp: MultiPolygon<T>) -> Self {
        Self(monotone_subdivision(mp.0))
    }
}

impl<T: GeoNum> Intersects<Coord<T>> for MonotonicPolygons<T> {
    fn intersects(&self, other: &Coord<T>) -> bool {
        self.0.iter().any(|mp| mp.intersects(other))
    }
}

#[cfg(test)]
mod tests;
