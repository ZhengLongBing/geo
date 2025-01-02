use super::{Distance, Euclidean};
use crate::algorithm::Intersects;
use crate::coordinate_position::{coord_pos_relative_to_ring, CoordPos};
use crate::geometry::*;
use crate::{CoordFloat, GeoFloat, GeoNum};
use num_traits::{Bounded, Float};
use rstar::primitives::CachedEnvelope;
use rstar::RTree;

// 距离是一个对称操作，因此我们可以为两者实现一次
macro_rules! symmetric_distance_impl {
    ($t:ident, $a:ty, $b:ty) => {
        impl<F> $crate::Distance<F, $a, $b> for Euclidean
        where
            F: $t,
        {
            fn distance(a: $a, b: $b) -> F {
                Self::distance(b, a)
            }
        }
    };
}

// ┌───────────────────────────┐
// │ Coord 的实现             │
// └───────────────────────────┘

impl<F: CoordFloat> Distance<F, Coord<F>, Coord<F>> for Euclidean {
    fn distance(origin: Coord<F>, destination: Coord<F>) -> F {
        let delta = origin - destination;
        delta.x.hypot(delta.y)
    }
}
impl<F: CoordFloat> Distance<F, Coord<F>, &Line<F>> for Euclidean {
    fn distance(coord: Coord<F>, line: &Line<F>) -> F {
        ::geo_types::private_utils::point_line_euclidean_distance(Point(coord), *line)
    }
}

// ┌───────────────────────────┐
// │ Point 的实现             │
// └───────────────────────────┘

/// 计算两个点之间的欧几里得距离（即毕达哥拉斯距离）
impl<F: CoordFloat> Distance<F, Point<F>, Point<F>> for Euclidean {
    /// 计算两个点之间的欧几里得距离（即毕达哥拉斯距离）
    ///
    /// # 单位
    /// - `origin`, `destination`：点，x/y 的单位表示非角度单位
    ///    — 例如米或英里，而不是经纬度。对于经纬度点，请使用
    ///    [`Haversine`] 或 [`Geodesic`] [度量空间]。
    /// - 返回值：与 `origin` 和 `destination` 点单位相同的距离
    ///
    /// # 示例
    /// ```
    /// use geo::{Euclidean, Distance};
    /// use geo::Point;
    /// // 网络墨卡托坐标
    /// let new_york_city = Point::new(-8238310.24, 4942194.78);
    /// // 网络墨卡托坐标
    /// let london = Point::new(-14226.63, 6678077.70);
    /// let distance: f64 = Euclidean::distance(new_york_city, london);
    ///
    /// assert_eq!(
    ///     8_405_286., // 网络墨卡托坐标下的米数
    ///     distance.round()
    /// );
    /// ```
    ///
    /// [`Haversine`]: crate::line_measures::metric_spaces::Haversine
    /// [`Geodesic`]: crate::line_measures::metric_spaces::Geodesic
    /// [metric spaces]: crate::line_measures::metric_spaces
    fn distance(origin: Point<F>, destination: Point<F>) -> F {
        Self::distance(origin.0, destination.0)
    }
}

impl<F: CoordFloat> Distance<F, &Point<F>, &Point<F>> for Euclidean {
    fn distance(origin: &Point<F>, destination: &Point<F>) -> F {
        Self::distance(*origin, *destination)
    }
}

impl<F: CoordFloat> Distance<F, &Point<F>, &Line<F>> for Euclidean {
    fn distance(origin: &Point<F>, destination: &Line<F>) -> F {
        geo_types::private_utils::point_line_euclidean_distance(*origin, *destination)
    }
}

impl<F: CoordFloat> Distance<F, &Point<F>, &LineString<F>> for Euclidean {
    fn distance(origin: &Point<F>, destination: &LineString<F>) -> F {
        geo_types::private_utils::point_line_string_euclidean_distance(*origin, destination)
    }
}

impl<F: GeoFloat> Distance<F, &Point<F>, &Polygon<F>> for Euclidean {
    fn distance(point: &Point<F>, polygon: &Polygon<F>) -> F {
        // 如果多边形与点相交或长度为零，则无需继续
        if polygon.exterior().0.is_empty() || polygon.intersects(point) {
            return F::zero();
        }
        // 折叠最小的内部环距离（如果有），然后是外部
        // 壳距离，返回两者之间的最小值
        polygon
            .interiors()
            .iter()
            .map(|ring| Self::distance(point, ring))
            .fold(Bounded::max_value(), |accum: F, val| accum.min(val))
            .min(
                polygon
                    .exterior()
                    .lines()
                    .map(|line| {
                        ::geo_types::private_utils::line_segment_distance(
                            point.0, line.start, line.end,
                        )
                    })
                    .fold(Bounded::max_value(), |accum, val| accum.min(val)),
            )
    }
}

// ┌──────────────────────────┐
// │ Line 的实现              │
// └──────────────────────────┘

symmetric_distance_impl!(CoordFloat, &Line<F>, Coord<F>);
symmetric_distance_impl!(CoordFloat, &Line<F>, &Point<F>);

impl<F: GeoFloat> Distance<F, &Line<F>, &Line<F>> for Euclidean {
    fn distance(line_a: &Line<F>, line_b: &Line<F>) -> F {
        if line_a.intersects(line_b) {
            return F::zero();
        }
        // 所有点到线的最小距离
        Self::distance(&line_a.start_point(), line_b)
            .min(Self::distance(&line_a.end_point(), line_b))
            .min(Self::distance(&line_b.start_point(), line_a))
            .min(Self::distance(&line_b.end_point(), line_a))
    }
}

impl<F: GeoFloat> Distance<F, &Line<F>, &LineString<F>> for Euclidean {
    fn distance(line: &Line<F>, line_string: &LineString<F>) -> F {
        line_string
            .lines()
            .fold(Bounded::max_value(), |acc, segment| {
                acc.min(Self::distance(line, &segment))
            })
    }
}

impl<F: GeoFloat> Distance<F, &Line<F>, &Polygon<F>> for Euclidean {
    fn distance(line: &Line<F>, polygon: &Polygon<F>) -> F {
        if line.intersects(polygon) {
            return F::zero();
        }

        // 注意：该实现稍有变化。
        std::iter::once(polygon.exterior())
            .chain(polygon.interiors().iter())
            .fold(Bounded::max_value(), |acc, line_string| {
                acc.min(Self::distance(line, line_string))
            })
    }
}

// ┌────────────────────────────────┐
// │ LineString 的实现              │
// └────────────────────────────────┘

symmetric_distance_impl!(CoordFloat, &LineString<F>, &Point<F>);
symmetric_distance_impl!(GeoFloat, &LineString<F>, &Line<F>);

impl<F: GeoFloat> Distance<F, &LineString<F>, &LineString<F>> for Euclidean {
    fn distance(line_string_a: &LineString<F>, line_string_b: &LineString<F>) -> F {
        if line_string_a.intersects(line_string_b) {
            F::zero()
        } else {
            nearest_neighbour_distance(line_string_a, line_string_b)
        }
    }
}

impl<F: GeoFloat> Distance<F, &LineString<F>, &Polygon<F>> for Euclidean {
    fn distance(line_string: &LineString<F>, polygon: &Polygon<F>) -> F {
        if line_string.intersects(polygon) {
            F::zero()
        } else if !polygon.interiors().is_empty()
            // 注意：在空的 line_string 上会爆炸
            && ring_contains_coord(polygon.exterior(), line_string.0[0])
        {
            // 检查每个环的距离，返回最小值
            let mut mindist: F = Float::max_value();
            for ring in polygon.interiors() {
                mindist = mindist.min(nearest_neighbour_distance(line_string, ring))
            }
            mindist
        } else {
            nearest_neighbour_distance(line_string, polygon.exterior())
        }
    }
}

// ┌─────────────────────────────┐
// │ Polygon 的实现              │
// └─────────────────────────────┘

symmetric_distance_impl!(GeoFloat, &Polygon<F>, &Point<F>);
symmetric_distance_impl!(GeoFloat, &Polygon<F>, &Line<F>);
symmetric_distance_impl!(GeoFloat, &Polygon<F>, &LineString<F>);

impl<F: GeoFloat> Distance<F, &Polygon<F>, &Polygon<F>> for Euclidean {
    fn distance(polygon_a: &Polygon<F>, polygon_b: &Polygon<F>) -> F {
        if polygon_a.intersects(polygon_b) {
            return F::zero();
        }
        // 注意：polygon_b.exterior() 为空时会爆炸
        // 包含检查
        if !polygon_a.interiors().is_empty()
            && ring_contains_coord(polygon_a.exterior(), polygon_b.exterior().0[0])
        {
            // 检查每个环的距离，返回最小值
            let mut mindist: F = Float::max_value();
            for ring in polygon_a.interiors() {
                mindist = mindist.min(nearest_neighbour_distance(polygon_b.exterior(), ring))
            }
            return mindist;
        } else if !polygon_b.interiors().is_empty()
            // 注意：polygon_a.exterior() 为空时会爆炸
            && ring_contains_coord(polygon_b.exterior(), polygon_a.exterior().0[0])
        {
            let mut mindist: F = Float::max_value();
            for ring in polygon_b.interiors() {
                mindist = mindist.min(nearest_neighbour_distance(polygon_a.exterior(), ring))
            }
            return mindist;
        }
        nearest_neighbour_distance(polygon_a.exterior(), polygon_b.exterior())
    }
}

// ┌────────────────────────────────────────┐
// │ Rect 和 Triangle 的实现                │
// └────────────────────────────────────────┘

/// 通过将三角形和矩形转换为多边形来实现欧几里得距离。
macro_rules! impl_euclidean_distance_for_polygonlike_geometry {
    ($polygonlike:ty,  [$($geometry_b:ty),*]) => {
        impl<F: GeoFloat> Distance<F, $polygonlike, $polygonlike> for Euclidean
        {
            fn distance(origin: $polygonlike, destination: $polygonlike) -> F {
                Self::distance(&origin.to_polygon(), destination)
            }
        }
        $(
            impl<F: GeoFloat> Distance<F, $polygonlike, $geometry_b> for Euclidean
            {
                fn distance(polygonlike: $polygonlike, geometry_b: $geometry_b) -> F {
                      Self::distance(&polygonlike.to_polygon(), geometry_b)
                }
            }
            symmetric_distance_impl!(GeoFloat, $geometry_b, $polygonlike);
        )*
    };
}

impl_euclidean_distance_for_polygonlike_geometry!(&Triangle<F>,  [&Point<F>, &Line<F>, &LineString<F>, &Polygon<F>, &Rect<F>]);
impl_euclidean_distance_for_polygonlike_geometry!(&Rect<F>,      [&Point<F>, &Line<F>, &LineString<F>, &Polygon<F>]);

// ┌───────────────────────────────────────────┐
// │ 多种几何类型的实现                        │
// └───────────────────────────────────────────┘

/// 多种几何类型的欧几里得距离实现。
macro_rules! impl_euclidean_distance_for_iter_geometry {
    ($iter_geometry:ty,  [$($to_geometry:ty),*]) => {
        impl<F: GeoFloat> Distance<F, $iter_geometry, $iter_geometry> for Euclidean {
            fn distance(origin: $iter_geometry, destination: $iter_geometry) -> F {
                origin
                    .iter()
                    .fold(Bounded::max_value(), |accum: F, member| {
                        accum.min(Self::distance(member, destination))
                    })
                }
        }
        $(
            impl<F: GeoFloat> Distance<F, $iter_geometry, $to_geometry> for Euclidean {
                fn distance(iter_geometry: $iter_geometry, to_geometry: $to_geometry) -> F {
                    iter_geometry
                        .iter()
                        .fold(Bounded::max_value(), |accum: F, member| {
                            accum.min(Self::distance(member, to_geometry))
                        })
                }
            }
            symmetric_distance_impl!(GeoFloat, $to_geometry, $iter_geometry);
        )*
  };
}

impl_euclidean_distance_for_iter_geometry!(&MultiPoint<F>,         [&Point<F>, &Line<F>, &LineString<F>, &MultiLineString<F>, &Polygon<F>, &MultiPolygon<F>, &GeometryCollection<F>, &Rect<F>, &Triangle<F>]);
impl_euclidean_distance_for_iter_geometry!(&MultiLineString<F>,    [&Point<F>, &Line<F>, &LineString<F>,                      &Polygon<F>, &MultiPolygon<F>, &GeometryCollection<F>, &Rect<F>, &Triangle<F>]);
impl_euclidean_distance_for_iter_geometry!(&MultiPolygon<F>,       [&Point<F>, &Line<F>, &LineString<F>,                      &Polygon<F>,                   &GeometryCollection<F>, &Rect<F>, &Triangle<F>]);
impl_euclidean_distance_for_iter_geometry!(&GeometryCollection<F>, [&Point<F>, &Line<F>, &LineString<F>,                      &Polygon<F>,                                           &Rect<F>, &Triangle<F>]);

// ┌──────────────────────────────┐
// │ Geometry 的实现             │
// └──────────────────────────────┘

/// 为 Geometry<T> 的每个具体几何类型实现欧几里得距离。
macro_rules! impl_euclidean_distance_for_geometry_and_variant {
    ([$($target:ty),*]) => {
        $(
            impl<F: GeoFloat> Distance<F, $target, &Geometry<F>> for Euclidean {
                fn distance(origin: $target, destination: &Geometry<F>) -> F {
                    match destination {
                        Geometry::Point(point) => Self::distance(origin, point),
                        Geometry::Line(line) => Self::distance(origin, line),
                        Geometry::LineString(line_string) => Self::distance(origin, line_string),
                        Geometry::Polygon(polygon) => Self::distance(origin, polygon),
                        Geometry::MultiPoint(multi_point) => Self::distance(origin, multi_point),
                        Geometry::MultiLineString(multi_line_string) => Self::distance(origin, multi_line_string),
                        Geometry::MultiPolygon(multi_polygon) => Self::distance(origin, multi_polygon),
                        Geometry::GeometryCollection(geometry_collection) => Self::distance(origin, geometry_collection),
                        Geometry::Rect(rect) => Self::distance(origin, rect),
                        Geometry::Triangle(triangle) => Self::distance(origin, triangle),
                    }
                }
            }
            symmetric_distance_impl!(GeoFloat, &Geometry<F>, $target);
        )*
    };
}

impl_euclidean_distance_for_geometry_and_variant!([&Point<F>, &MultiPoint<F>, &Line<F>, &LineString<F>, &MultiLineString<F>, &Polygon<F>, &MultiPolygon<F>, &Triangle<F>, &Rect<F>, &GeometryCollection<F>]);

impl<F: GeoFloat> Distance<F, &Geometry<F>, &Geometry<F>> for Euclidean {
    fn distance(origin: &Geometry<F>, destination: &Geometry<F>) -> F {
        match origin {
            Geometry::Point(point) => Self::distance(point, destination),
            Geometry::Line(line) => Self::distance(line, destination),
            Geometry::LineString(line_string) => Self::distance(line_string, destination),
            Geometry::Polygon(polygon) => Self::distance(polygon, destination),
            Geometry::MultiPoint(multi_point) => Self::distance(multi_point, destination),
            Geometry::MultiLineString(multi_line_string) => {
                Self::distance(multi_line_string, destination)
            }
            Geometry::MultiPolygon(multi_polygon) => Self::distance(multi_polygon, destination),
            Geometry::GeometryCollection(geometry_collection) => {
                Self::distance(geometry_collection, destination)
            }
            Geometry::Rect(rect) => Self::distance(rect, destination),
            Geometry::Triangle(triangle) => Self::distance(triangle, destination),
        }
    }
}

// ┌───────────────────────────┐
// │ 实现的工具               │
// └───────────────────────────┘

/// 使用 R* 树和最近邻查找计算最小距离
// 这有点慢且内存效率低，但肯定比平方时间要好
fn nearest_neighbour_distance<F: GeoFloat>(geom1: &LineString<F>, geom2: &LineString<F>) -> F {
    let tree_a = RTree::bulk_load(geom1.lines().map(CachedEnvelope::new).collect());
    let tree_b = RTree::bulk_load(geom2.lines().map(CachedEnvelope::new).collect());
    // 返回所有几何 a 点与几何 b 线，以及所有几何 b 点与几何 a 线之间的最小距离
    geom2
        .points()
        .fold(Bounded::max_value(), |acc: F, point| {
            let nearest = tree_a.nearest_neighbor(&point).unwrap();
            acc.min(Euclidean::distance(nearest as &Line<F>, &point))
        })
        .min(geom1.points().fold(Bounded::max_value(), |acc, point| {
            let nearest = tree_b.nearest_neighbor(&point).unwrap();
            acc.min(Euclidean::distance(nearest as &Line<F>, &point))
        }))
}

fn ring_contains_coord<T: GeoNum>(ring: &LineString<T>, c: Coord<T>) -> bool {
    match coord_pos_relative_to_ring(c, ring) {
        CoordPos::Inside => true,
        CoordPos::OnBoundary | CoordPos::Outside => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::orient::{Direction, Orient};
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
        // 结果与 Shapely 相符
        assert_relative_eq!(dist, 2.0485900789263356);
        assert_relative_eq!(dist2, 1.118033988749895);
        assert_relative_eq!(dist3, std::f64::consts::SQRT_2); // 解决 clippy::correctness 错误 approx_constant (1.4142135623730951)
        assert_relative_eq!(dist4, 1.5811388300841898);
        // 点在直线上
        let zero_dist = line_segment_distance(p1, p1, p2);
        assert_relative_eq!(zero_dist, 0.0);
    }
    #[test]
    // 点到多边形，点在外部
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
        // 八边形外部的随机点
        let p = Point::new(2.5, 0.5);
        let dist = Euclidean::distance(&p, &poly);
        assert_relative_eq!(dist, 2.1213203435596424);
    }
    #[test]
    // 点到多边形，点在内部
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
        // 八边形内部的随机点
        let p = Point::new(5.5, 2.1);
        let dist = Euclidean::distance(&p, &poly);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // 点到多边形，点在边界上
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
        // 八边形上的一个点
        let p = Point::new(5.0, 1.0);
        let dist = Euclidean::distance(&p, &poly);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // 点到多边形，点在边界上
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
        assert_relative_eq!(Euclidean::distance(&poly, &bugged_point), 0.);
    }
    #[test]
    // 点到多边形，空多边形
    fn point_polygon_empty_test() {
        // 一个空多边形
        let points = vec![];
        let ls = LineString::new(points);
        let poly = Polygon::new(ls, vec![]);
        // 八边形上的一个点
        let p = Point::new(2.5, 0.5);
        let dist = Euclidean::distance(&p, &poly);
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
        // 从八边形内切割出一个三角形
        let int_points = vec![(3.5, 3.5), (4.4, 1.5), (2.6, 1.5), (3.5, 3.5)];
        let ls_ext = LineString::from(ext_points);
        let ls_int = LineString::from(int_points);
        let poly = Polygon::new(ls_ext, vec![ls_int]);
        // 切割三角形内的一个点
        let p = Point::new(3.5, 2.5);
        let dist = Euclidean::distance(&p, &poly);

        // 0.41036467732879783 <-- 来自 Shapely
        assert_relative_eq!(dist, 0.41036467732879767);
    }

    #[test]
    fn line_distance_multipolygon_do_not_intersect_test() {
        // 检查来自多多边形的距离
        // 等于从最近多边形来的距离
        // 无论该距离是多少
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
        let dist_mp_ln = Euclidean::distance(&ln, &mp);
        let dist_pol1_ln = Euclidean::distance(&ln, &pol1);
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
        assert_relative_eq!(Euclidean::distance(&p, &mp), 60.959002616512684);
    }
    #[test]
    // 点到线串
    fn point_linestring_distance_test() {
        // 类似八边形，但缺少最低的水平段
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
        // 线串内部的随机点
        let p = Point::new(5.5, 2.1);
        let dist = Euclidean::distance(&p, &ls);
        assert_relative_eq!(dist, 1.1313708498984762);
    }
    #[test]
    // 点到线串，点位于线串上
    fn point_linestring_contains_test() {
        // 类似八边形，但缺少最低的水平段
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
        // 位于线串上的点
        let p = Point::new(5.0, 4.0);
        let dist = Euclidean::distance(&p, &ls);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // 点到线串，封闭三角形
    fn point_linestring_triangle_test() {
        let points = vec![(3.5, 3.5), (4.4, 2.0), (2.6, 2.0), (3.5, 3.5)];
        let ls = LineString::from(points);
        let p = Point::new(3.5, 2.5);
        let dist = Euclidean::distance(&p, &ls);
        assert_relative_eq!(dist, 0.5);
    }
    #[test]
    // 点到线串，空线串
    fn point_linestring_empty_test() {
        let points = vec![];
        let ls = LineString::new(points);
        let p = Point::new(5.0, 4.0);
        let dist = Euclidean::distance(&p, &ls);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    fn distance_multilinestring_test() {
        let v1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0)]);
        let v2 = LineString::from(vec![(1.0, 10.0), (2.0, 0.0), (3.0, 1.0)]);
        let mls = MultiLineString::new(vec![v1, v2]);
        let p = Point::new(50.0, 50.0);
        assert_relative_eq!(Euclidean::distance(&p, &mls), 63.25345840347388);
    }
    #[test]
    fn distance1_test() {
        assert_relative_eq!(
            Euclidean::distance(&Point::new(0., 0.), &Point::new(1., 0.)),
            1.
        );
    }
    #[test]
    fn distance2_test() {
        let dist =
            Euclidean::distance(&Point::new(-72.1235, 42.3521), &Point::new(72.1260, 70.612));
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
        assert_relative_eq!(Euclidean::distance(&p, &mp), 64.03124237432849)
    }
    #[test]
    fn distance_line_test() {
        let line0 = Line::from([(0., 0.), (5., 0.)]);
        let p0 = Point::new(2., 3.);
        let p1 = Point::new(3., 0.);
        let p2 = Point::new(6., 0.);
        assert_relative_eq!(Euclidean::distance(&line0, &p0), 3.);
        assert_relative_eq!(Euclidean::distance(&p0, &line0), 3.);

        assert_relative_eq!(Euclidean::distance(&line0, &p1), 0.);
        assert_relative_eq!(Euclidean::distance(&p1, &line0), 0.);

        assert_relative_eq!(Euclidean::distance(&line0, &p2), 1.);
        assert_relative_eq!(Euclidean::distance(&p2, &line0), 1.);
    }
    #[test]
    fn distance_line_line_test() {
        let line0 = Line::from([(0., 0.), (5., 0.)]);
        let line1 = Line::from([(2., 1.), (7., 2.)]);
        assert_relative_eq!(Euclidean::distance(&line0, &line1), 1.);
        assert_relative_eq!(Euclidean::distance(&line1, &line0), 1.);
    }
    #[test]
    // 参见 https://github.com/georust/geo/issues/476
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
        assert_eq!(Euclidean::distance(&line, &poly), 0.18752558079168907);
    }
    #[test]
    // 测试边-顶点最小距离
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
    // 测试顶点-顶点最小距离
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
    // 测试边-边最小距离
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
        let distance = Euclidean::distance(&poly1, &poly2);
        // GEOS 的结果是 2.2864896295566055
        assert_relative_eq!(distance, 2.2864896295566055);
    }
    #[test]
    // 一个多边形位于另一个多边形的环内；它们在 DE-9IM 意义上是独立的：FF2FF1212
    fn test_poly_in_ring() {
        let shell = geo_test_fixtures::shell::<f64>();
        let ring = geo_test_fixtures::ring::<f64>();
        let poly_in_ring = geo_test_fixtures::poly_in_ring::<f64>();
        // 内部的多边形“在”外部环内，但它们是独立的
        let outside = Polygon::new(shell, vec![ring]);
        let inside = Polygon::new(poly_in_ring, vec![]);
        assert_relative_eq!(Euclidean::distance(&outside, &inside), 5.992772737231033);
    }
    #[test]
    // 两条环形线串；一个包围另一个，但它们既不接触也不相交
    fn test_linestring_distance() {
        let ring = geo_test_fixtures::ring::<f64>();
        let poly_in_ring = geo_test_fixtures::poly_in_ring::<f64>();
        assert_relative_eq!(Euclidean::distance(&ring, &poly_in_ring), 5.992772737231033);
    }
    #[test]
    // 线到多边形测试：多边形上的最近点不是最接近线端点的点
    fn test_line_polygon_simple() {
        let line = Line::from([(0.0, 0.0), (0.0, 3.0)]);
        let v = vec![(5.0, 1.0), (5.0, 2.0), (0.25, 1.5), (5.0, 1.0)];
        let poly = Polygon::new(v.into(), vec![]);
        assert_relative_eq!(Euclidean::distance(&line, &poly), 0.25);
    }
    #[test]
    // 线到多边形测试：线与多边形相交
    fn test_line_polygon_intersects() {
        let line = Line::from([(0.5, 0.0), (0.0, 3.0)]);
        let v = vec![(5.0, 1.0), (5.0, 2.0), (0.25, 1.5), (5.0, 1.0)];
        let poly = Polygon::new(v.into(), vec![]);
        assert_relative_eq!(Euclidean::distance(&line, &poly), 0.0);
    }
    #[test]
    // 线到多边形测试：线位于内部环内
    fn test_line_polygon_inside_ring() {
        let line = Line::from([(4.4, 1.5), (4.45, 1.5)]);
        let v = vec![(5.0, 1.0), (5.0, 2.0), (0.25, 1.0), (5.0, 1.0)];
        let v2 = vec![(4.5, 1.2), (4.5, 1.8), (3.5, 1.2), (4.5, 1.2)];
        let poly = Polygon::new(v.into(), vec![v2.into()]);
        assert_relative_eq!(Euclidean::distance(&line, &poly), 0.04999999999999982);
    }
    #[test]
    // 线串到线的测试
    fn test_linestring_line_distance() {
        let line = Line::from([(0.0, 0.0), (0.0, 2.0)]);
        let ls: LineString<_> = vec![(3.0, 0.0), (1.0, 1.0), (3.0, 2.0)].into();
        assert_relative_eq!(Euclidean::distance(&ls, &line), 1.0);
    }

    #[test]
    // 三角形到点的测试：点在顶点上
    fn test_triangle_point_on_vertex_distance() {
        let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let point = Point::new(0.0, 0.0);
        assert_relative_eq!(Euclidean::distance(&triangle, &point), 0.0);
    }

    #[test]
    // 三角形到点的测试：点在边上
    fn test_triangle_point_on_edge_distance() {
        let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let point = Point::new(1.5, 0.0);
        assert_relative_eq!(Euclidean::distance(&triangle, &point), 0.0);
    }

    #[test]
    // 三角形到点的测试
    fn test_triangle_point_distance() {
        let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let point = Point::new(2.0, 3.0);
        assert_relative_eq!(Euclidean::distance(&triangle, &point), 1.0);
    }

    #[test]
    // 三角形到点的测试：点在三角形内
    fn test_triangle_point_inside_distance() {
        let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let point = Point::new(1.0, 0.5);
        assert_relative_eq!(Euclidean::distance(&triangle, &point), 0.0);
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
            Euclidean::distance(&first_polygon, &second_polygon),
            224.35357967013238
        );
    }
    #[test]
    fn fast_path_regression() {
        // 如果未修复的情况下重新引入快速路径算法，该测试将失败
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
        assert_eq!(Euclidean::distance(&p1, &p2), 50.0f64);
        assert_eq!(Euclidean::distance(&p3, &p4), 50.0f64);
        assert_eq!(Euclidean::distance(&p1, &p4), 50.0f64);
        assert_eq!(Euclidean::distance(&p2, &p3), 50.0f64);
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
        assert_relative_eq!(Euclidean::distance(&test_p, &gc), 60.959002616512684);

        let test_multipoint = MultiPoint::new(vec![test_p]);
        assert_relative_eq!(
            Euclidean::distance(&test_multipoint, &gc),
            60.959002616512684
        );

        let test_line = Line::from([(50., 50.), (60., 60.)]);
        assert_relative_eq!(Euclidean::distance(&test_line, &gc), 60.959002616512684);

        let test_ls = LineString::from(vec![(50., 50.), (60., 60.), (70., 70.)]);
        assert_relative_eq!(Euclidean::distance(&test_ls, &gc), 60.959002616512684);

        let test_mls = MultiLineString::new(vec![test_ls]);
        assert_relative_eq!(Euclidean::distance(&test_mls, &gc), 60.959002616512684);

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
        assert_relative_eq!(Euclidean::distance(&test_poly, &gc), 60.959002616512684);

        let test_multipoly = MultiPolygon::new(vec![test_poly]);
        assert_relative_eq!(
            Euclidean::distance(&test_multipoly, &gc),
            60.959002616512684
        );

        let test_tri = Triangle::from([(50., 50.), (60., 50.), (55., 55.)]);
        assert_relative_eq!(Euclidean::distance(&test_tri, &gc), 60.959002616512684);

        let test_rect = Rect::new(coord! { x: 50., y: 50. }, coord! { x: 60., y: 60. });
        assert_relative_eq!(Euclidean::distance(&test_rect, &gc), 60.959002616512684);

        let test_gc = GeometryCollection(vec![Geometry::Rect(test_rect)]);
        assert_relative_eq!(Euclidean::distance(&test_gc, &gc), 60.959002616512684);
    }
}
