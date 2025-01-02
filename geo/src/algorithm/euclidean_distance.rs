use crate::{
    Coord, GeoFloat, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};
use crate::{Distance, Euclidean};
use num_traits::{float::FloatConst, Bounded, Signed};

use rstar::primitives::CachedEnvelope;
use rstar::RTree;
use rstar::RTreeNum;

#[deprecated(
    since = "0.29.0",
    note = "请使用 `Euclidean::distance` 方法代替 `Distance` 特性中的方法"
)]
/// 返回两个几何形状之间的距离。
pub trait EuclideanDistance<T, Rhs = Self> {
    /// 返回两个几何形状之间的距离
    ///
    /// 如果`Point`包含在`Polygon`中，距离是`0.0`
    ///
    /// 如果`Point`位于`Polygon`的外环或内环上，距离是`0.0`
    ///
    /// 如果`Point`位于`LineString`上，距离是`0.0`
    ///
    /// `Point`和空`LineString`之间的距离是`0.0`
    ///
    /// # 示例
    ///
    /// `Point`到`Point`：
    ///
    /// ```
    /// use approx::assert_relative_eq;
    /// use geo::EuclideanDistance;
    /// use geo::point;
    ///
    /// let p1 = point!(x: -72.1235, y: 42.3521);
    /// let p2 = point!(x: -72.1260, y: 42.45);
    ///
    /// let distance = p1.euclidean_distance(&p2);
    ///
    /// assert_relative_eq!(distance, 0.09793191512474639);
    /// ```
    ///
    /// `Point`到`Polygon`：
    ///
    /// ```
    /// use approx::assert_relative_eq;
    /// use geo::EuclideanDistance;
    /// use geo::{point, polygon};
    ///
    /// let polygon = polygon![
    ///     (x: 5., y: 1.),
    ///     (x: 4., y: 2.),
    ///     (x: 4., y: 3.),
    ///     (x: 5., y: 4.),
    ///     (x: 6., y: 4.),
    ///     (x: 7., y: 3.),
    ///     (x: 7., y: 2.),
    ///     (x: 6., y: 1.),
    ///     (x: 5., y: 1.),
    /// ];
    ///
    /// let point = point!(x: 2.5, y: 0.5);
    ///
    /// let distance = point.euclidean_distance(&polygon);
    ///
    /// assert_relative_eq!(distance, 2.1213203435596424);
    /// ```
    ///
    /// `Point`到`LineString`：
    ///
    /// ```
    /// use approx::assert_relative_eq;
    /// use geo::EuclideanDistance;
    /// use geo::{point, line_string};
    ///
    /// let line_string = line_string![
    ///     (x: 5., y: 1.),
    ///     (x: 4., y: 2.),
    ///     (x: 4., y: 3.),
    ///     (x: 5., y: 4.),
    ///     (x: 6., y: 4.),
    ///     (x: 7., y: 3.),
    ///     (x: 7., y: 2.),
    ///     (x: 6., y: 1.),
    /// ];
    ///
    /// let point = point!(x: 5.5, y: 2.1);
    ///
    /// let distance = point.euclidean_distance(&line_string);
    ///
    /// assert_relative_eq!(distance, 1.1313708498984762);
    /// ```
    fn euclidean_distance(&self, rhs: &Rhs) -> T;
}

// ┌───────────────────────────┐
// │ Coord 的实现 │
// └───────────────────────────┘
#[allow(deprecated)]
impl<T> EuclideanDistance<T, Coord<T>> for Coord<T>
where
    T: GeoFloat,
{
    /// 两个`Coord`之间的最小距离
    fn euclidean_distance(&self, c: &Coord<T>) -> T {
        Euclidean::distance(Point(*self), Point(*c))
    }
}

#[allow(deprecated)]
impl<T> EuclideanDistance<T, Line<T>> for Coord<T>
where
    T: GeoFloat,
{
    /// 从`Coord`到`Line`的最小距离
    fn euclidean_distance(&self, line: &Line<T>) -> T {
        Euclidean::distance(&Point(*self), line)
    }
}

// ┌───────────────────────────┐
// │ Point 的实现 │
// └───────────────────────────┘

#[allow(deprecated)]
impl<T> EuclideanDistance<T, Point<T>> for Point<T>
where
    T: GeoFloat,
{
    /// 两个点之间的最小距离
    fn euclidean_distance(&self, p: &Point<T>) -> T {
        Euclidean::distance(*self, *p)
    }
}

#[allow(deprecated)]
impl<T> EuclideanDistance<T, Line<T>> for Point<T>
where
    T: GeoFloat,
{
    /// 从`Line`到`Point`的最小距离
    fn euclidean_distance(&self, line: &Line<T>) -> T {
        Euclidean::distance(self, line)
    }
}

#[allow(deprecated)]
impl<T> EuclideanDistance<T, LineString<T>> for Point<T>
where
    T: GeoFloat,
{
    /// 从`Point`到`LineString`的最小距离
    fn euclidean_distance(&self, line_string: &LineString<T>) -> T {
        Euclidean::distance(self, line_string)
    }
}

#[allow(deprecated)]
impl<T> EuclideanDistance<T, Polygon<T>> for Point<T>
where
    T: GeoFloat,
{
    /// 从`Point`到`Polygon`的最小距离
    fn euclidean_distance(&self, polygon: &Polygon<T>) -> T {
        Euclidean::distance(self, polygon)
    }
}

// ┌──────────────────────────┐
// │ Line 的实现 │
// └──────────────────────────┘

#[allow(deprecated)]
impl<T> EuclideanDistance<T, Coord<T>> for Line<T>
where
    T: GeoFloat,
{
    /// 从`Line`到`Coord`的最小距离
    fn euclidean_distance(&self, coord: &Coord<T>) -> T {
        Euclidean::distance(self, *coord)
    }
}

#[allow(deprecated)]
impl<T> EuclideanDistance<T, Point<T>> for Line<T>
where
    T: GeoFloat,
{
    /// 从`Line`到`Point`的最小距离
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        Euclidean::distance(self, point)
    }
}

#[allow(deprecated)]
/// Line 到 Line 的距离
impl<T> EuclideanDistance<T, Line<T>> for Line<T>
where
    T: GeoFloat + FloatConst + Signed + RTreeNum,
{
    fn euclidean_distance(&self, other: &Line<T>) -> T {
        Euclidean::distance(self, other)
    }
}

#[allow(deprecated)]
/// Line 到 LineString
impl<T> EuclideanDistance<T, LineString<T>> for Line<T>
where
    T: GeoFloat + FloatConst + Signed + RTreeNum,
{
    fn euclidean_distance(&self, other: &LineString<T>) -> T {
        Euclidean::distance(self, other)
    }
}

#[allow(deprecated)]
// Line 到 Polygon 的距离
impl<T> EuclideanDistance<T, Polygon<T>> for Line<T>
where
    T: GeoFloat + Signed + RTreeNum + FloatConst,
{
    fn euclidean_distance(&self, other: &Polygon<T>) -> T {
        Euclidean::distance(self, other)
    }
}

// ┌────────────────────────────────┐
// │ LineString 的实现 │
// └────────────────────────────────┘

#[allow(deprecated)]
impl<T> EuclideanDistance<T, Point<T>> for LineString<T>
where
    T: GeoFloat,
{
    /// 从`LineString`到`Point`的最小距离
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        Euclidean::distance(self, point)
    }
}

#[allow(deprecated)]
/// LineString 到 Line
impl<T> EuclideanDistance<T, Line<T>> for LineString<T>
where
    T: GeoFloat + FloatConst + Signed + RTreeNum,
{
    fn euclidean_distance(&self, other: &Line<T>) -> T {
        Euclidean::distance(self, other)
    }
}

#[allow(deprecated)]
/// LineString 到 LineString 的距离
impl<T> EuclideanDistance<T, LineString<T>> for LineString<T>
where
    T: GeoFloat + Signed + RTreeNum,
{
    fn euclidean_distance(&self, other: &LineString<T>) -> T {
        Euclidean::distance(self, other)
    }
}

#[allow(deprecated)]
/// LineString 到 Polygon
impl<T> EuclideanDistance<T, Polygon<T>> for LineString<T>
where
    T: GeoFloat + FloatConst + Signed + RTreeNum,
{
    fn euclidean_distance(&self, other: &Polygon<T>) -> T {
        Euclidean::distance(self, other)
    }
}

// ┌─────────────────────────────┐
// │ Polygon 的实现 │
// └─────────────────────────────┘

#[allow(deprecated)]
impl<T> EuclideanDistance<T, Point<T>> for Polygon<T>
where
    T: GeoFloat,
{
    /// 从`Polygon`到`Point`的最小距离
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        Euclidean::distance(self, point)
    }
}

#[allow(deprecated)]
// Polygon 到 Line 的距离
impl<T> EuclideanDistance<T, Line<T>> for Polygon<T>
where
    T: GeoFloat + FloatConst + Signed + RTreeNum,
{
    fn euclidean_distance(&self, other: &Line<T>) -> T {
        Euclidean::distance(self, other)
    }
}

#[allow(deprecated)]
/// Polygon 到 LineString 的距离
impl<T> EuclideanDistance<T, LineString<T>> for Polygon<T>
where
    T: GeoFloat + FloatConst + Signed + RTreeNum,
{
    fn euclidean_distance(&self, other: &LineString<T>) -> T {
        Euclidean::distance(self, other)
    }
}

#[allow(deprecated)]
// Polygon 到 Polygon 的距离
impl<T> EuclideanDistance<T, Polygon<T>> for Polygon<T>
where
    T: GeoFloat + FloatConst + RTreeNum,
{
    fn euclidean_distance(&self, poly2: &Polygon<T>) -> T {
        Euclidean::distance(self, poly2)
    }
}

// ┌────────────────────────────────────────┐
// │ Rect 和 Triangle 的实现  │
// └────────────────────────────────────────┘

/// 通过将三角形和矩形转换为多边形来实现两者的欧几里得距离。
macro_rules! impl_euclidean_distance_for_polygonlike_geometry {
  ($for:ty,  [$($target:ty),*]) => {
      $(
          #[allow(deprecated)]
          impl<T> EuclideanDistance<T, $target> for $for
          where
              T: GeoFloat + Signed + RTreeNum + FloatConst,
          {
              fn euclidean_distance(&self, other: &$target) -> T {
                  Euclidean::distance(self, other)
              }
          }
      )*
  };
}

impl_euclidean_distance_for_polygonlike_geometry!(Triangle<T>,  [Point<T>, MultiPoint<T>, Line<T>, LineString<T>, MultiLineString<T>, Polygon<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_euclidean_distance_for_polygonlike_geometry!(Rect<T>,      [Point<T>, MultiPoint<T>, Line<T>, LineString<T>, MultiLineString<T>, Polygon<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);

/// 通过将三角形或矩形转换为多边形来实现其他几何类型到三角形和矩形的欧几里得距离。
macro_rules! impl_euclidean_distance_to_polygonlike_geometry {
  ($for:ty,  [$($target:ty),*]) => {
      $(
          #[allow(deprecated)]
          impl<T> EuclideanDistance<T, $target> for $for
          where
              T: GeoFloat + Signed + RTreeNum + FloatConst,
          {
              fn euclidean_distance(&self, other: &$target) -> T {
                  Euclidean::distance(self, other)
              }
          }
      )*
  };
}

impl_euclidean_distance_to_polygonlike_geometry!(Point<T>,               [Rect<T>, Triangle<T>]);
impl_euclidean_distance_to_polygonlike_geometry!(MultiPoint<T>,          [Rect<T>, Triangle<T>]);
impl_euclidean_distance_to_polygonlike_geometry!(Line<T>,                [Rect<T>, Triangle<T>]);
impl_euclidean_distance_to_polygonlike_geometry!(LineString<T>,          [Rect<T>, Triangle<T>]);
impl_euclidean_distance_to_polygonlike_geometry!(MultiLineString<T>,     [Rect<T>, Triangle<T>]);
impl_euclidean_distance_to_polygonlike_geometry!(Polygon<T>,             [Rect<T>, Triangle<T>]);
impl_euclidean_distance_to_polygonlike_geometry!(MultiPolygon<T>,        [Rect<T>, Triangle<T>]);
impl_euclidean_distance_to_polygonlike_geometry!(GeometryCollection<T>,  [Rect<T>, Triangle<T>]);

// ┌───────────────────────────────────────────┐
// │ 多几何类型的实现  │
// └───────────────────────────────────────────┘

/// 多几何类型的欧几里得距离实现。
macro_rules! impl_euclidean_distance_for_iter_geometry {
  ($for:ty,  [$($target:ty),*]) => {
      $(
          #[allow(deprecated)]
          impl<T> EuclideanDistance<T, $target> for $for
          where
              T: GeoFloat + FloatConst + RTreeNum,
          {
              fn euclidean_distance(&self, target: &$target) -> T {
                  Euclidean::distance(self, target)
              }
          }
      )*
  };
}

impl_euclidean_distance_for_iter_geometry!(MultiPoint<T>,         [Point<T>, MultiPoint<T>, Line<T>, LineString<T>, MultiLineString<T>, Polygon<T>, MultiPolygon<T>, GeometryCollection<T>]);
impl_euclidean_distance_for_iter_geometry!(MultiLineString<T>,    [Point<T>, MultiPoint<T>, Line<T>, LineString<T>, MultiLineString<T>, Polygon<T>, MultiPolygon<T>, GeometryCollection<T>]);
impl_euclidean_distance_for_iter_geometry!(MultiPolygon<T>,       [Point<T>, MultiPoint<T>, Line<T>, LineString<T>, MultiLineString<T>, Polygon<T>, MultiPolygon<T>, GeometryCollection<T>]);
impl_euclidean_distance_for_iter_geometry!(GeometryCollection<T>, [Point<T>, MultiPoint<T>, Line<T>, LineString<T>, MultiLineString<T>, Polygon<T>, MultiPolygon<T>, GeometryCollection<T>]);

/// 其他几何类型到多几何类型的欧几里得距离实现，
/// 使用多几何类型的实现。
macro_rules! impl_euclidean_distance_from_iter_geometry {
  ($for:ty,  [$($target:ty),*]) => {
      $(
        #[allow(deprecated)]
        impl<T> EuclideanDistance<T, $target> for $for
        where
            T: GeoFloat + FloatConst + RTreeNum
        {
          fn euclidean_distance(&self, target: &$target) -> T {
              Euclidean::distance(self, target)
          }
        }
      )*
  };
}

// 此宏用于为非多几何类型实现 EuclideanDistance 于多几何类型。
// Rect 和 Triangle 被省略，因为这些实现已包括在 Rect 和 Triangle 部分。
impl_euclidean_distance_from_iter_geometry!(Point<T>,         [MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>]);
impl_euclidean_distance_from_iter_geometry!(Line<T>,          [MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>]);
impl_euclidean_distance_from_iter_geometry!(LineString<T>,    [MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>]);
impl_euclidean_distance_from_iter_geometry!(Polygon<T>,       [MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>]);

// ┌─────────────────────────────────────────────────────────┐
// │ 为每个几何类型实现到 Geometry<T> 的实现   │
// └─────────────────────────────────────────────────────────┘

/// 为每个具体的 Geometry 类型实现到 Geometry<T> 的欧几里得距离。
macro_rules! impl_euclidean_distance_to_geometry_for_specific {
  ([$($for:ty),*]) => {
      $(
          #[allow(deprecated)]
          impl<T> EuclideanDistance<T, Geometry<T>> for $for
          where
              T: GeoFloat + FloatConst + RTreeNum,
          {
              fn euclidean_distance(&self, geom: &Geometry<T>) -> T {
                Euclidean::distance(self, geom)
              }
          }
      )*
  };
}

impl_euclidean_distance_to_geometry_for_specific!([Point<T>, MultiPoint<T>, Line<T>, LineString<T>, MultiLineString<T>, Polygon<T>, MultiPolygon<T>, Triangle<T>, Rect<T>, GeometryCollection<T>]);

// ┌──────────────────────────────┐
// │ 对 Geometry 的实现  │
// └──────────────────────────────┘

/// Geometry<T> 到每个具体的 Geometry 类型的欧几里得距离实现。
macro_rules! impl_euclidean_distance_to_specific_for_geometry {
  ([$($for:ty),*]) => {
      $(
          #[allow(deprecated)]
          impl<T> EuclideanDistance<T, $for> for Geometry<T>
          where
              T: GeoFloat + FloatConst + RTreeNum
          {
              crate::geometry_delegate_impl! {
                  fn euclidean_distance(&self, other: &$for) -> T;
              }
          }
      )*
  };
}

impl_euclidean_distance_to_specific_for_geometry!([Point<T>, MultiPoint<T>, Line<T>, LineString<T>, MultiLineString<T>, Polygon<T>, MultiPolygon<T>, Triangle<T>, Rect<T>, GeometryCollection<T>]);

#[allow(deprecated)]
impl<T> EuclideanDistance<T> for Geometry<T>
where
    T: GeoFloat + FloatConst,
{
    crate::geometry_delegate_impl! {
        fn euclidean_distance(&self, other: &Geometry<T>) -> T;
    }
}

// ┌───────────┐
// │ 工具函数 │
// └───────────┘

#[deprecated(
    since = "0.29.0",
    note = "请使用 `Euclidean::distance` 方法代替 `Distance` 特性中的方法"
)]
/// 使用 R* 树和最近邻居查找计算最小距离
// 这有点慢且内存效率低，但肯定比二次时间复杂度好
pub fn nearest_neighbour_distance<T>(geom1: &LineString<T>, geom2: &LineString<T>) -> T
where
    T: GeoFloat + RTreeNum,
{
    let tree_a = RTree::bulk_load(geom1.lines().map(CachedEnvelope::new).collect());
    let tree_b = RTree::bulk_load(geom2.lines().map(CachedEnvelope::new).collect());
    // 返回所有几何 a 点和几何 b 线、以及所有几何 b 点和几何 a 线之间的最小距离
    geom2
        .points()
        .fold(<T as Bounded>::max_value(), |acc, point| {
            let nearest = tree_a.nearest_neighbor(&point).unwrap();
            #[allow(deprecated)]
            acc.min(nearest.euclidean_distance(&point))
        })
        .min(geom1.points().fold(Bounded::max_value(), |acc, point| {
            let nearest = tree_b.nearest_neighbor(&point).unwrap();
            #[allow(deprecated)]
            acc.min(nearest.euclidean_distance(&point))
        }))
}

#[cfg(test)]
mod test {
    // 这些测试已移植到新的 line_measures::euclidean::distance 模块。
    // 它们最终会被移除，所以让我们暂时保留这些弃用的测试
    // 作为保证旧特性已正确委托给新实现的保证。
    #![allow(deprecated)]

    use super::*;
    use crate::orient::Direction;
    use crate::Orient;
    use crate::{Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
    use geo_types::{coord, polygon, private_utils::line_segment_distance};

    #[test]
    fn line_segment_distance_test() {
        let o1 = Point::new(8.0, 0.0);
        let o2 = Point::new(5.5, 0.0);
        let o3 = Point::new(5.0, 0.0);
        let o4 = Point::new(4.5, 1.5);

        let p1 = Point::new(7.2, 2.0);
        let p2 = Point::new(6.0, 1.0);

        let dist = line_segment_distance(o1, p1, p2);
        let dist2 = line_segment_distance(o2, p1, p2);
        let dist3 = line_segment_distance(o3, p1, p2);
        let dist4 = line_segment_distance(o4, p1, p2);
        // 结果与 Shapely一致
        assert_relative_eq!(dist, 2.0485900789263356);
        assert_relative_eq!(dist2, 1.118033988749895);
        assert_relative_eq!(dist3, std::f64::consts::SQRT_2); // 解决	clippy::correctness error approx_constant (1.4142135623730951)
        assert_relative_eq!(dist4, 1.5811388300841898);
        // 点在直线上
        let zero_dist = line_segment_distance(p1, p1, p2);
        assert_relative_eq!(zero_dist, 0.0);
    }
    #[test]
    // 点到多边形，外部点
    fn point_polygon_distance_outside_test() {
        // 一个八边形
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.),
        ];
        let ls = LineString::from(points);
        let poly = Polygon::new(ls, vec![]);
        // 一个八边形外的随机点
        let p = Point::new(2.5, 0.5);
        let dist = p.euclidean_distance(&poly);
        assert_relative_eq!(dist, 2.1213203435596424);
    }
    #[test]
    // 点到多边形，内部点
    fn point_polygon_distance_inside_test() {
        // 一个八边形
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.),
        ];
        let ls = LineString::from(points);
        let poly = Polygon::new(ls, vec![]);
        // 一个八边形内部的随机点
        let p = Point::new(5.5, 2.1);
        let dist = p.euclidean_distance(&poly);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // 点到多边形，边界上的点
    fn point_polygon_distance_boundary_test() {
        // 一个八边形
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.),
        ];
        let ls = LineString::from(points);
        let poly = Polygon::new(ls, vec![]);
        // 一个八边形上的点
        let p = Point::new(5.0, 1.0);
        let dist = p.euclidean_distance(&poly);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // 点到多边形，边界上的点
    fn point_polygon_boundary_test2() {
        let exterior = LineString::from(vec![
            (0., 0.),
            (0., 0.0004),
            (0.0004, 0.0004),
            (0.0004, 0.),
            (0., 0.),
        ]);

        let poly = Polygon::new(exterior, vec![]);
        let bugged_point = Point::new(0.0001, 0.);
        assert_relative_eq!(poly.euclidean_distance(&bugged_point), 0.);
    }
    #[test]
    // 点到多边形，空多边形
    fn point_polygon_empty_test() {
        // 一个空多边形
        let points = vec![];
        let ls = LineString::new(points);
        let poly = Polygon::new(ls, vec![]);
        // 一个八边形上的点
        let p = Point::new(2.5, 0.5);
        let dist = p.euclidean_distance(&poly);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // 点到多边形，有内部环
    fn point_polygon_interior_cutout_test() {
        // 一个八边形
        let ext_points = vec![
            (4., 1.),
            (5., 2.),
            (5., 3.),
            (4., 4.),
            (3., 4.),
            (2., 3.),
            (2., 2.),
            (3., 1.),
            (4., 1.),
        ];
        // 在八边形内刨去一个三角形
        let int_points = vec![(3.5, 3.5), (4.4, 1.5), (2.6, 1.5), (3.5, 3.5)];
        let ls_ext = LineString::from(ext_points);
        let ls_int = LineString::from(int_points);
        let poly = Polygon::new(ls_ext, vec![ls_int]);
        // 内部切除区域中的一个点
        let p = Point::new(3.5, 2.5);
        let dist = p.euclidean_distance(&poly);

        // 0.41036467732879783 <-- Shapely
        assert_relative_eq!(dist, 0.41036467732879767);
    }

    #[test]
    fn line_distance_multipolygon_do_not_intersect_test() {
        // 检查从多边形的距离
        // 等于从最近多边形的距离
        // 单独看，这个距离是什么
        let ls1 = LineString::from(vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (5.0, 15.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ]);
        let ls2 = LineString::from(vec![
            (0.0, 30.0),
            (0.0, 25.0),
            (10.0, 25.0),
            (10.0, 30.0),
            (0.0, 30.0),
        ]);
        let ls3 = LineString::from(vec![
            (15.0, 30.0),
            (15.0, 25.0),
            (20.0, 25.0),
            (20.0, 30.0),
            (15.0, 30.0),
        ]);
        let pol1 = Polygon::new(ls1, vec![]);
        let pol2 = Polygon::new(ls2, vec![]);
        let pol3 = Polygon::new(ls3, vec![]);
        let mp = MultiPolygon::new(vec![pol1.clone(), pol2, pol3]);
        let pnt1 = Point::new(0.0, 15.0);
        let pnt2 = Point::new(10.0, 20.0);
        let ln = Line::new(pnt1.0, pnt2.0);
        let dist_mp_ln = ln.euclidean_distance(&mp);
        let dist_pol1_ln = ln.euclidean_distance(&pol1);
        assert_relative_eq!(dist_mp_ln, dist_pol1_ln);
    }

    #[test]
    fn point_distance_multipolygon_test() {
        let ls1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 0.0), (0.0, 0.0)]);
        let ls2 = LineString::from(vec![(3.0, 0.0), (4.0, 10.0), (5.0, 0.0), (3.0, 0.0)]);
        let p1 = Polygon::new(ls1, vec![]);
        let p2 = Polygon::new(ls2, vec![]);
        let mp = MultiPolygon::new(vec![p1, p2]);
        let p = Point::new(50.0, 50.0);
        assert_relative_eq!(p.euclidean_distance(&mp), 60.959002616512684);
    }
    #[test]
    // 点到LineString
    fn point_linestring_distance_test() {
        // 类似于一个八边形，但缺少最低的水平段
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
        ];
        let ls = LineString::from(points);
        // 在`LineString`内的随机点
        let p = Point::new(5.5, 2.1);
        let dist = p.euclidean_distance(&ls);
        assert_relative_eq!(dist, 1.1313708498984762);
    }
    #[test]
    // 点到 LineString， 点在`LineString`上
    fn point_linestring_contains_test() {
        // 类似于一个八边形，但缺少最低的水平段
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
        ];
        let ls = LineString::from(points);
        // 在`LineString`上的点
        let p = Point::new(5.0, 4.0);
        let dist = p.euclidean_distance(&ls);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // 点到 LineString，封闭三角形
    fn point_linestring_triangle_test() {
        let points = vec![(3.5, 3.5), (4.4, 2.0), (2.6, 2.0), (3.5, 3.5)];
        let ls = LineString::from(points);
        let p = Point::new(3.5, 2.5);
        let dist = p.euclidean_distance(&ls);
        assert_relative_eq!(dist, 0.5);
    }
    #[test]
    // 点到 LineString，空`LineString`
    fn point_linestring_empty_test() {
        let points = vec![];
        let ls = LineString::new(points);
        let p = Point::new(5.0, 4.0);
        let dist = p.euclidean_distance(&ls);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    fn distance_multilinestring_test() {
        let v1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0)]);
        let v2 = LineString::from(vec![(1.0, 10.0), (2.0, 0.0), (3.0, 1.0)]);
        let mls = MultiLineString::new(vec![v1, v2]);
        let p = Point::new(50.0, 50.0);
        assert_relative_eq!(p.euclidean_distance(&mls), 63.25345840347388);
    }
    #[test]
    fn distance1_test() {
        assert_relative_eq!(
            Point::new(0., 0.).euclidean_distance(&Point::new(1., 0.)),
            1.
        );
    }
    #[test]
    fn distance2_test() {
        let dist = Point::new(-72.1235, 42.3521).euclidean_distance(&Point::new(72.1260, 70.612));
        assert_relative_eq!(dist, 146.99163308930207);
    }
    #[test]
    fn distance_multipoint_test() {
        let v = vec![
            Point::new(0.0, 10.0),
            Point::new(1.0, 1.0),
            Point::new(10.0, 0.0),
            Point::new(1.0, -1.0),
            Point::new(0.0, -10.0),
            Point::new(-1.0, -1.0),
            Point::new(-10.0, 0.0),
            Point::new(-1.0, 1.0),
            Point::new(0.0, 10.0),
        ];
        let mp = MultiPoint::new(v);
        let p = Point::new(50.0, 50.0);
        assert_relative_eq!(p.euclidean_distance(&mp), 64.03124237432849)
    }
    #[test]
    fn distance_line_test() {
        let line0 = Line::from([(0., 0.), (5., 0.)]);
        let p0 = Point::new(2., 3.);
        let p1 = Point::new(3., 0.);
        let p2 = Point::new(6., 0.);
        assert_relative_eq!(line0.euclidean_distance(&p0), 3.);
        assert_relative_eq!(p0.euclidean_distance(&line0), 3.);

        assert_relative_eq!(line0.euclidean_distance(&p1), 0.);
        assert_relative_eq!(p1.euclidean_distance(&line0), 0.);

        assert_relative_eq!(line0.euclidean_distance(&p2), 1.);
        assert_relative_eq!(p2.euclidean_distance(&line0), 1.);
    }
    #[test]
    fn distance_line_line_test() {
        let line0 = Line::from([(0., 0.), (5., 0.)]);
        let line1 = Line::from([(2., 1.), (7., 2.)]);
        assert_relative_eq!(line0.euclidean_distance(&line1), 1.);
        assert_relative_eq!(line1.euclidean_distance(&line0), 1.);
    }
    #[test]
    // 见 https://github.com/georust/geo/issues/476
    fn distance_line_polygon_test() {
        let line = Line::new(
            coord! {
                x: -0.17084137691985102,
                y: 0.8748085493016657,
            },
            coord! {
                x: -0.17084137691985102,
                y: 0.09858870312437906,
            },
        );
        let poly: Polygon<f64> = polygon![
            coord! {
                x: -0.10781391405721802,
                y: -0.15433610862574643,
            },
            coord! {
                x: -0.7855276236615211,
                y: 0.23694208404779793,
            },
            coord! {
                x: -0.7855276236615214,
                y: -0.5456143012992907,
            },
            coord! {
                x: -0.10781391405721802,
                y: -0.15433610862574643,
            },
        ];
        assert_eq!(line.euclidean_distance(&poly), 0.18752558079168907);
    }
    #[test]
    // 测试边缘顶点最小距离
    fn test_minimum_polygon_distance() {
        let points_raw = [
            (126., 232.),
            (126., 212.),
            (112., 202.),
            (97., 204.),
            (87., 215.),
            (87., 232.),
            (100., 246.),
            (118., 247.),
        ];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString::from(points), vec![]);

        let points_raw_2 = [
            (188., 231.),
            (189., 207.),
            (174., 196.),
            (164., 196.),
            (147., 220.),
            (158., 242.),
            (177., 242.),
        ];
        let points2 = points_raw_2
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly2 = Polygon::new(LineString::from(points2), vec![]);
        let dist = nearest_neighbour_distance(poly1.exterior(), poly2.exterior());
        assert_relative_eq!(dist, 21.0);
    }
    #[test]
    // 测试顶点顶点最小距离
    fn test_minimum_polygon_distance_2() {
        let points_raw = [
            (118., 200.),
            (153., 179.),
            (106., 155.),
            (88., 190.),
            (118., 200.),
        ];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString::from(points), vec![]);

        let points_raw_2 = [
            (242., 186.),
            (260., 146.),
            (182., 175.),
            (216., 193.),
            (242., 186.),
        ];
        let points2 = points_raw_2
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly2 = Polygon::new(LineString::from(points2), vec![]);
        let dist = nearest_neighbour_distance(poly1.exterior(), poly2.exterior());
        assert_relative_eq!(dist, 29.274562336608895);
    }
    #[test]
    // 测试边缘边缘最小距离
    fn test_minimum_polygon_distance_3() {
        let points_raw = [
            (182., 182.),
            (182., 168.),
            (138., 160.),
            (136., 193.),
            (182., 182.),
        ];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString::from(points), vec![]);

        let points_raw_2 = [
            (232., 196.),
            (234., 150.),
            (194., 165.),
            (194., 191.),
            (232., 196.),
        ];
        let points2 = points_raw_2
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly2 = Polygon::new(LineString::from(points2), vec![]);
        let dist = nearest_neighbour_distance(poly1.exterior(), poly2.exterior());
        assert_relative_eq!(dist, 12.0);
    }
    #[test]
    fn test_large_polygon_distance() {
        let ls = geo_test_fixtures::norway_main::<f64>();
        let poly1 = Polygon::new(ls, vec![]);
        let vec2 = vec![
            (4.921875, 66.33750501996518),
            (3.69140625, 65.21989393613207),
            (6.15234375, 65.07213008560697),
            (4.921875, 66.33750501996518),
        ];
        let poly2 = Polygon::new(vec2.into(), vec![]);
        let distance = poly1.euclidean_distance(&poly2);
        // GEOS 表示 2.2864896295566055
        assert_relative_eq!(distance, 2.2864896295566055);
    }
    #[test]
    // 一个多边形在另一个多边形的环内；它们在 DE-9IM 意义上是不相交的：
    // FF2FF1212
    fn test_poly_in_ring() {
        let shell = geo_test_fixtures::shell::<f64>();
        let ring = geo_test_fixtures::ring::<f64>();
        let poly_in_ring = geo_test_fixtures::poly_in_ring::<f64>();
        // 内部在外部环的内部，但它们是分离的
        let outside = Polygon::new(shell, vec![ring]);
        let inside = Polygon::new(poly_in_ring, vec![]);
        assert_relative_eq!(outside.euclidean_distance(&inside), 5.992772737231033);
    }
    #[test]
    // 两个环形 LineStrings；一个包含另一个，但它们既不接触也不相交
    fn test_linestring_distance() {
        let ring = geo_test_fixtures::ring::<f64>();
        let poly_in_ring = geo_test_fixtures::poly_in_ring::<f64>();
        assert_relative_eq!(ring.euclidean_distance(&poly_in_ring), 5.992772737231033);
    }
    #[test]
    // Line-Polygon测试：多边形上的最近点不是到一个Line端点的最近点
    fn test_line_polygon_simple() {
        let line = Line::from([(0.0, 0.0), (0.0, 3.0)]);
        let v = vec![(5.0, 1.0), (5.0, 2.0), (0.25, 1.5), (5.0, 1.0)];
        let poly = Polygon::new(v.into(), vec![]);
        assert_relative_eq!(line.euclidean_distance(&poly), 0.25);
    }
    #[test]
    // Line-Polygon测试：Line交多边形
    fn test_line_polygon_intersects() {
        let line = Line::from([(0.5, 0.0), (0.0, 3.0)]);
        let v = vec![(5.0, 1.0), (5.0, 2.0), (0.25, 1.5), (5.0, 1.0)];
        let poly = Polygon::new(v.into(), vec![]);
        assert_relative_eq!(line.euclidean_distance(&poly), 0.0);
    }
    #[test]
    // Line-Polygon测试：Line包含在内部环中
    fn test_line_polygon_inside_ring() {
        let line = Line::from([(4.4, 1.5), (4.45, 1.5)]);
        let v = vec![(5.0, 1.0), (5.0, 2.0), (0.25, 1.0), (5.0, 1.0)];
        let v2 = vec![(4.5, 1.2), (4.5, 1.8), (3.5, 1.2), (4.5, 1.2)];
        let poly = Polygon::new(v.into(), vec![v2.into()]);
        assert_relative_eq!(line.euclidean_distance(&poly), 0.04999999999999982);
    }
    #[test]
    // LineString-Line测试
    fn test_linestring_line_distance() {
        let line = Line::from([(0.0, 0.0), (0.0, 2.0)]);
        let ls: LineString<_> = vec![(3.0, 0.0), (1.0, 1.0), (3.0, 2.0)].into();
        assert_relative_eq!(ls.euclidean_distance(&line), 1.0);
    }

    #[test]
    // Triangle-Point测试：点在顶点上
    fn test_triangle_point_on_vertex_distance() {
        let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let point = Point::new(0.0, 0.0);
        assert_relative_eq!(triangle.euclidean_distance(&point), 0.0);
    }

    #[test]
    // Triangle-Point测试：点在边缘
    fn test_triangle_point_on_edge_distance() {
        let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let point = Point::new(1.5, 0.0);
        assert_relative_eq!(triangle.euclidean_distance(&point), 0.0);
    }

    #[test]
    // Triangle-Point测试
    fn test_triangle_point_distance() {
        let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let point = Point::new(2.0, 3.0);
        assert_relative_eq!(triangle.euclidean_distance(&point), 1.0);
    }

    #[test]
    // Triangle-Point测试：点在三角形内部
    fn test_triangle_point_inside_distance() {
        let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let point = Point::new(1.0, 0.5);
        assert_relative_eq!(triangle.euclidean_distance(&point), 0.0);
    }

    #[test]
    fn convex_and_nearest_neighbour_comparison() {
        let ls1: LineString<f64> = vec![
            Coord::from((57.39453770777941, 307.60533608924663)),
            Coord::from((67.1800355576469, 309.6654408997451)),
            Coord::from((84.89693692793338, 225.5101593908847)),
            Coord::from((75.1114390780659, 223.45005458038628)),
            Coord::from((57.39453770777941, 307.60533608924663)),
        ]
        .into();
        let first_polygon: Polygon<f64> = Polygon::new(ls1, vec![]);
        let ls2: LineString<f64> = vec![
            Coord::from((138.11769866645008, -45.75134112915392)),
            Coord::from((130.50230476949187, -39.270154833870336)),
            Coord::from((184.94426964987397, 24.699153900578573)),
            Coord::from((192.55966354683218, 18.217967605294987)),
            Coord::from((138.11769866645008, -45.75134112915392)),
        ]
        .into();
        let second_polygon = Polygon::new(ls2, vec![]);

        assert_relative_eq!(
            first_polygon.euclidean_distance(&second_polygon),
            224.35357967013238
        );
    }
    #[test]
    fn fast_path_regression() {
        // 如果重新引入快速路径算法而未修复，该测试将失败
        let p1 = polygon!(
            (x: 0_f64, y: 0_f64),
            (x: 300_f64, y: 0_f64),
            (x: 300_f64, y: 100_f64),
            (x: 0_f64, y: 100_f64),
        )
        .orient(Direction::Default);
        let p2 = polygon!(
            (x: 100_f64, y: 150_f64),
            (x: 150_f64, y: 200_f64),
            (x: 50_f64, y: 200_f64),
        )
        .orient(Direction::Default);
        let p3 = polygon!(
            (x: 0_f64, y: 0_f64),
            (x: 300_f64, y: 0_f64),
            (x: 300_f64, y: 100_f64),
            (x: 0_f64, y: 100_f64),
        )
        .orient(Direction::Reversed);
        let p4 = polygon!(
            (x: 100_f64, y: 150_f64),
            (x: 150_f64, y: 200_f64),
            (x: 50_f64, y: 200_f64),
        )
        .orient(Direction::Reversed);
        assert_eq!(p1.euclidean_distance(&p2), 50.0f64);
        assert_eq!(p3.euclidean_distance(&p4), 50.0f64);
        assert_eq!(p1.euclidean_distance(&p4), 50.0f64);
        assert_eq!(p2.euclidean_distance(&p3), 50.0f64);
    }
    #[test]
    fn all_types_geometry_collection_test() {
        let p = Point::new(0.0, 0.0);
        let line = Line::from([(-1.0, -1.0), (-2.0, -2.0)]);
        let ls = LineString::from(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 0.0)]);
        let poly = Polygon::new(
            LineString::from(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 0.0), (0.0, 0.0)]),
            vec![],
        );
        let tri = Triangle::from([(0.0, 0.0), (1.0, 10.0), (2.0, 0.0)]);
        let rect = Rect::new((0.0, 0.0), (-1.0, -1.0));

        let ls1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 0.0), (0.0, 0.0)]);
        let ls2 = LineString::from(vec![(3.0, 0.0), (4.0, 10.0), (5.0, 0.0), (3.0, 0.0)]);
        let p1 = Polygon::new(ls1, vec![]);
        let p2 = Polygon::new(ls2, vec![]);
        let mpoly = MultiPolygon::new(vec![p1, p2]);

        let v = vec![
            Point::new(0.0, 10.0),
            Point::new(1.0, 1.0),
            Point::new(10.0, 0.0),
            Point::new(1.0, -1.0),
            Point::new(0.0, -10.0),
            Point::new(-1.0, -1.0),
            Point::new(-10.0, 0.0),
            Point::new(-1.0, 1.0),
            Point::new(0.0, 10.0),
        ];
        let mpoint = MultiPoint::new(v);

        let v1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0)]);
        let v2 = LineString::from(vec![(1.0, 10.0), (2.0, 0.0), (3.0, 1.0)]);
        let mls = MultiLineString::new(vec![v1, v2]);

        let gc = GeometryCollection(vec![
            Geometry::Point(p),
            Geometry::Line(line),
            Geometry::LineString(ls),
            Geometry::Polygon(poly),
            Geometry::MultiPoint(mpoint),
            Geometry::MultiLineString(mls),
            Geometry::MultiPolygon(mpoly),
            Geometry::Triangle(tri),
            Geometry::Rect(rect),
        ]);

        let test_p = Point::new(50., 50.);
        assert_relative_eq!(test_p.euclidean_distance(&gc), 60.959002616512684);

        let test_multipoint = MultiPoint::new(vec![test_p]);
        assert_relative_eq!(test_multipoint.euclidean_distance(&gc), 60.959002616512684);

        let test_line = Line::from([(50., 50.), (60., 60.)]);
        assert_relative_eq!(test_line.euclidean_distance(&gc), 60.959002616512684);

        let test_ls = LineString::from(vec![(50., 50.), (60., 60.), (70., 70.)]);
        assert_relative_eq!(test_ls.euclidean_distance(&gc), 60.959002616512684);

        let test_mls = MultiLineString::new(vec![test_ls]);
        assert_relative_eq!(test_mls.euclidean_distance(&gc), 60.959002616512684);

        let test_poly = Polygon::new(
            LineString::from(vec![
                (50., 50.),
                (60., 50.),
                (60., 60.),
                (55., 55.),
                (50., 50.),
            ]),
            vec![],
        );
        assert_relative_eq!(test_poly.euclidean_distance(&gc), 60.959002616512684);

        let test_multipoly = MultiPolygon::new(vec![test_poly]);
        assert_relative_eq!(test_multipoly.euclidean_distance(&gc), 60.959002616512684);

        let test_tri = Triangle::from([(50., 50.), (60., 50.), (55., 55.)]);
        assert_relative_eq!(test_tri.euclidean_distance(&gc), 60.959002616512684);

        let test_rect = Rect::new(coord! { x: 50., y: 50. }, coord! { x: 60., y: 60. });
        assert_relative_eq!(test_rect.euclidean_distance(&gc), 60.959002616512684);

        let test_gc = GeometryCollection(vec![Geometry::Rect(test_rect)]);
        assert_relative_eq!(test_gc.euclidean_distance(&gc), 60.959002616512684);
    }
}
