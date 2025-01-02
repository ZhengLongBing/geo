use std::cmp::{Ordering, Reverse};

use crate::algorithm::{
    bounding_rect::BoundingRect,
    centroid::Centroid,
    coords_iter::CoordsIter,
    dimensions::HasDimensions,
    line_intersection::LineIntersection,
    line_measures::{Distance, Euclidean},
    lines_iter::LinesIter,
    relate::Relate,
};
use crate::geometry::*;
use crate::sweep::{Intersections, SweepPoint};
use crate::GeoFloat;

/// 计算内部点。
///
/// 内部点是一个保证与给定几何体相交的点，并且在可能的情况下将严格位于几何体的内部，
/// 如果几何体没有面积，将在其边缘上。还将尽最大努力使该点尽可能位于几何体中央。
///
/// 对于多边形，此点通过绘制一条大致将多边形的边界框一分为二的直线，
/// 再与多边形相交，并计算由此相交产生的最长线段的中点来确定。
/// 对于线，如果线具有内部点，则返回最接近线的质心的非端点顶点，否则返回端点。
///
/// 对于多几何体或组合，计算组成部分的内部点，并返回其中的一个
/// （对于多多边形，它是上面描述的最长相交线段的中点；对于其他的，
/// 使用距离集合的质心最近的内部点）。
///
/// # 示例
///
/// ```
/// use geo::InteriorPoint;
/// use geo::{point, polygon};
///
/// // 菱形状的多边形
/// let polygon = polygon![
///     (x: -2., y: 1.),
///     (x: 1., y: 3.),
///     (x: 4., y: 1.),
///     (x: 1., y: -1.),
///     (x: -2., y: 1.),
/// ];
///
/// assert_eq!(
///     Some(point!(x: 1., y: 2.)),
///     polygon.interior_point(),
/// );
/// ```
pub trait InteriorPoint {
    type Output;

    /// 计算几何图形内部的代表点
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::InteriorPoint;
    /// use geo::{line_string, point};
    ///
    /// let line_string = line_string![
    ///     (x: 40.02f64, y: 116.34),
    ///     (x: 40.02f64, y: 118.23),
    ///     (x: 40.02f64, y: 120.15),
    /// ];
    ///
    /// assert_eq!(
    ///     Some(point!(x: 40.02, y: 118.23)),
    ///     line_string.interior_point(),
    /// );
    /// ```
    fn interior_point(&self) -> Self::Output;
}

impl<T> InteriorPoint for Line<T>
where
    T: GeoFloat,
{
    type Output = Point<T>;

    fn interior_point(&self) -> Self::Output {
        // 线段的中点由于浮点数的舍入问题，并不能保证与线分段有 `intersects()` 关系，请直接使用起点
        self.start_point()
    }
}

impl<T> InteriorPoint for LineString<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    // 如果有，则返回最接近质心的非端点顶点的LineString的内部点，否则返回起点
    fn interior_point(&self) -> Self::Output {
        match self.0.len() {
            0 => None,
            // 对于长度为2的LineString，计算的中点可能不在该线段上，故直接使用起点
            1 | 2 => Some(self.0[0].into()),
            _ => {
                let centroid = self.centroid().expect("非空的LineString期望存在质心");
                self.0[1..(self.0.len() - 1)]
                    .iter()
                    .map(|coord| {
                        let pt = Point::from(*coord);
                        (pt, Euclidean::distance(pt, centroid))
                    })
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Less))
                    .map(|(pt, _distance)| pt)
            }
        }
    }
}

impl<T> InteriorPoint for MultiLineString<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    /// MultiLineString中内部点是所有组成LineString的内部点中距离MultiLineString质心最近的一个
    fn interior_point(&self) -> Self::Output {
        if let Some(centroid) = self.centroid() {
            self.iter()
                .filter_map(|linestring| {
                    linestring
                        .interior_point()
                        .map(|pt| (pt, Euclidean::distance(pt, centroid)))
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Less))
                .map(|(pt, _distance)| pt)
        } else {
            None
        }
    }
}

fn polygon_interior_point_with_segment_length<T: GeoFloat>(
    polygon: &Polygon<T>,
) -> Option<(Point<T>, T)> {
    // 针对单个点的多边形的特殊情况，因为该算法无法支持这个情况
    if polygon.exterior().0.len() == 1 {
        return Some((polygon.exterior().0[0].into(), T::zero()));
    }

    let two = T::one() + T::one();

    let bounds = polygon.bounding_rect()?;

    // 使用边界的中点来扫描，除非它恰好与多边形上的某个顶点重合；
    // 如果是，则通过与下一个最接近中心的顶点的Y坐标取平均的方法对线稍作扰动，以减少共线相交的可能
    let mut y_mid = (bounds.min().y + bounds.max().y) / two;
    if polygon.coords_iter().any(|coord| coord.y == y_mid) {
        let next_closest = polygon
            .coords_iter()
            .filter_map(|coord| {
                if coord.y == y_mid {
                    None
                } else {
                    Some((coord.y, (coord.y - y_mid).abs()))
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Less));
        if let Some((closest, _)) = next_closest {
            y_mid = (y_mid + closest) / two
        }
    };

    let scan_line = Line::new(
        Coord {
            x: bounds.min().x,
            y: y_mid,
        },
        Coord {
            x: bounds.max().x,
            y: y_mid,
        },
    );

    let lines = polygon.lines_iter().chain(std::iter::once(scan_line));

    let mut intersections: Vec<SweepPoint<T>> = Vec::new();
    for (l1, l2, inter) in Intersections::from_iter(lines) {
        if !(l1 == scan_line || l2 == scan_line) {
            continue;
        }
        match inter {
            LineIntersection::Collinear { intersection } => {
                intersections.push(SweepPoint::from(intersection.start));
                intersections.push(SweepPoint::from(intersection.end));
            }
            LineIntersection::SinglePoint { intersection, .. } => {
                intersections.push(SweepPoint::from(intersection));
            }
        }
    }
    intersections.sort();

    let mut segments = Vec::new();
    let mut intersections_iter = intersections.iter().peekable();
    while let (Some(start), Some(end)) = (intersections_iter.next(), intersections_iter.peek()) {
        let length = end.x - start.x;
        let midpoint = Point::new((start.x + end.x) / two, y_mid);
        segments.push((midpoint, length));
    }
    segments.sort_by(|a, b| b.1.total_cmp(&a.1));

    for (midpoint, segment_length) in segments {
        // 东西方向相邻的点对将限定多边形内部的一些线段，和外部的一些线段；确认这是前者
        let relation = polygon.relate(&midpoint);
        if relation.is_intersects() {
            return Some((
                midpoint,
                if relation.is_contains() {
                    segment_length
                } else {
                    // 如果我们的点在边界上，必须是因为这是一个零面积的多边形
                    // 所以如果我们是从多多边形上下文中调用的， 我们希望这个选项比其他可能有非零面积的多边形被降级
                    T::zero()
                },
            ));
        }
    }
    // 如果我们到这里仍然没有成功，返回任何一个顶点，如果有的话
    polygon
        .coords_iter()
        .next()
        .map(|coord| (coord.into(), T::zero()))
}

impl<T> InteriorPoint for Polygon<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    fn interior_point(&self) -> Self::Output {
        polygon_interior_point_with_segment_length(self).map(|(point, _length)| point)
    }
}

impl<T> InteriorPoint for MultiPolygon<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    fn interior_point(&self) -> Self::Output {
        let segments = self
            .iter()
            .filter_map(polygon_interior_point_with_segment_length);
        segments
            .min_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Less))
            .map(|(point, _length)| point)
    }
}

impl<T> InteriorPoint for Rect<T>
where
    T: GeoFloat,
{
    type Output = Point<T>;

    fn interior_point(&self) -> Self::Output {
        self.center().into()
    }
}

impl<T> InteriorPoint for Point<T>
where
    T: GeoFloat,
{
    type Output = Point<T>;

    fn interior_point(&self) -> Self::Output {
        *self
    }
}

///
/// ```
/// use geo::InteriorPoint;
/// use geo::{MultiPoint, Point};
///
/// let empty: Vec<Point> = Vec::new();
/// let empty_multi_points: MultiPoint<_> = empty.into();
/// assert_eq!(empty_multi_points.interior_point(), None);
///
/// let points: MultiPoint<_> = vec![(5., 1.), (1., 3.), (3., 2.)].into();
/// assert_eq!(points.interior_point(), Some(Point::new(3., 2.)));
/// ```
impl<T> InteriorPoint for MultiPoint<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    fn interior_point(&self) -> Self::Output {
        if let Some(centroid) = self.centroid() {
            self.iter()
                .map(|pt| (pt, Euclidean::distance(pt, &centroid)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Less))
                .map(|(pt, _distance)| *pt)
        } else {
            None
        }
    }
}

impl<T> InteriorPoint for Geometry<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    crate::geometry_delegate_impl! {
        fn interior_point(&self) -> Self::Output;
    }
}

impl<T> InteriorPoint for GeometryCollection<T>
where
    T: GeoFloat,
{
    type Output = Option<Point<T>>;

    fn interior_point(&self) -> Self::Output {
        if let Some(centroid) = self.centroid() {
            self.iter()
                .filter_map(|geom| {
                    geom.interior_point().map(|pt| {
                        (
                            pt,
                            // 增大维度，减小距离
                            (
                                Reverse(geom.dimensions()),
                                Euclidean::distance(pt, centroid),
                            ),
                        )
                    })
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Less))
                .map(|(pt, _distance)| pt)
        } else {
            None
        }
    }
}

impl<T> InteriorPoint for Triangle<T>
where
    T: GeoFloat,
{
    type Output = Point<T>;

    fn interior_point(&self) -> Self::Output {
        self.centroid()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        algorithm::{contains::Contains, intersects::Intersects},
        coord, line_string, point, polygon,
    };

    /// 创建坐标的小工具
    fn c<T: GeoFloat>(x: T, y: T) -> Coord<T> {
        coord! { x: x, y: y }
    }

    /// 创建点的小工具
    fn p<T: GeoFloat>(x: T, y: T) -> Point<T> {
        point! { x: x, y: y }
    }

    // 测试：LineString的内部点
    #[test]
    fn empty_linestring_test() {
        let linestring: LineString<f32> = line_string![];
        let interior_point = linestring.interior_point();
        assert!(interior_point.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let coord = coord! {
            x: 40.02f64,
            y: 116.34,
        };
        let linestring = line_string![coord];
        let interior_point = linestring.interior_point();
        assert_eq!(interior_point, Some(Point::from(coord)));
    }
    #[test]
    fn linestring_test() {
        let linestring = line_string![
            (x: 1., y: 1.),
            (x: 7., y: 1.),
            (x: 8., y: 1.),
            (x: 9., y: 1.),
            (x: 10., y: 1.),
            (x: 11., y: 1.)
        ];
        assert_eq!(linestring.interior_point(), Some(point!(x: 7., y: 1. )));
    }
    #[test]
    fn linestring_with_repeated_point_test() {
        let l1 = LineString::from(vec![p(1., 1.), p(1., 1.), p(1., 1.)]);
        assert_eq!(l1.interior_point(), Some(p(1., 1.)));

        let l2 = LineString::from(vec![p(2., 2.), p(2., 2.), p(2., 2.)]);
        let mls = MultiLineString::new(vec![l1, l2]);
        assert_eq!(mls.interior_point(), Some(p(1., 1.)));
    }
    // 测试：MultiLineString的内部点
    #[test]
    fn empty_multilinestring_test() {
        let mls: MultiLineString = MultiLineString::new(vec![]);
        let interior_point = mls.interior_point();
        assert!(interior_point.is_none());
    }
    #[test]
    fn multilinestring_with_empty_line_test() {
        let mls: MultiLineString = MultiLineString::new(vec![line_string![]]);
        let interior_point = mls.interior_point();
        assert!(interior_point.is_none());
    }
    #[test]
    fn multilinestring_length_0_test() {
        let coord = coord! {
            x: 40.02f64,
            y: 116.34,
        };
        let mls: MultiLineString = MultiLineString::new(vec![
            line_string![coord],
            line_string![coord],
            line_string![coord],
        ]);
        assert_relative_eq!(mls.interior_point().unwrap(), Point::from(coord));
    }
    #[test]
    fn multilinestring_one_line_test() {
        let linestring = line_string![
            (x: 1., y: 1.),
            (x: 7., y: 1.),
            (x: 8., y: 1.),
            (x: 9., y: 1.),
            (x: 10., y: 1.),
            (x: 11., y: 1.)
        ];
        let mls: MultiLineString = MultiLineString::new(vec![linestring]);
        assert_relative_eq!(mls.interior_point().unwrap(), point! { x: 7., y: 1. });
    }
    #[test]
    fn multilinestring_test() {
        let v1 = line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 10.0)];
        let v2 = line_string![(x: 1.0, y: 10.0), (x: 2.0, y: 0.0), (x: 3.0, y: 1.0)];
        let v3 = line_string![(x: -12.0, y: -100.0), (x: 7.0, y: 8.0)];
        let mls = MultiLineString::new(vec![v1, v2, v3]);
        assert_eq!(mls.interior_point().unwrap(), point![x: 0., y: 0.]);
    }
    // 测试：Polygon的内部点
    #[test]
    fn empty_polygon_test() {
        let poly: Polygon<f32> = polygon![];
        assert!(poly.interior_point().is_none());
    }
    #[test]
    fn polygon_one_point_test() {
        let p = point![ x: 2., y: 1. ];
        let v = Vec::new();
        let linestring = line_string![p.0];
        let poly = Polygon::new(linestring, v);
        assert_relative_eq!(poly.interior_point().unwrap(), p);
    }

    #[test]
    fn polygon_test() {
        let poly = polygon![
            (x: 0., y: 0.),
            (x: 2., y: 0.),
            (x: 2., y: 2.),
            (x: 0., y: 2.),
            (x: 0., y: 0.)
        ];
        assert_relative_eq!(poly.interior_point().unwrap(), point![x:1., y:1.]);
    }
    #[test]
    fn polygon_hole_test() {
        // 六边形
        let ls1 = LineString::from(vec![
            (5.0, 1.0),
            (4.0, 2.0),
            (4.0, 3.0),
            (5.0, 4.0),
            (6.0, 4.0),
            (7.0, 3.0),
            (7.0, 2.0),
            (6.0, 1.0),
            (5.0, 1.0),
        ]);

        let ls2 = LineString::from(vec![(5.0, 1.3), (5.5, 2.0), (6.0, 1.3), (5.0, 1.3)]);

        let ls3 = LineString::from(vec![(5., 2.3), (5.5, 3.0), (6., 2.3), (5., 2.3)]);

        let p1 = Polygon::new(ls1, vec![ls2, ls3]);
        let interior_point = p1.interior_point().unwrap();
        assert!(p1.contains(&interior_point));
        assert_relative_eq!(interior_point, point!(x: 4.571428571428571, y: 2.5));
    }
    #[test]
    fn flat_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![p(0., 1.), p(1., 1.), p(0., 1.)]),
            vec![],
        );
        assert_eq!(poly.interior_point(), Some(p(0.5, 1.)));
    }
    #[test]
    fn diagonal_flat_polygon_test() {
        // 常规的相交方法恰好不会产生与多边形相交的点，因此测试回退到顶点
        let start: Coord<f64> = Coord {
            x: 0.632690318327692,
            y: 0.08104532928154995,
        };
        let end: Coord<f64> = Coord {
            x: 0.4685039949468325,
            y: 0.31750332644855794,
        };
        let poly = Polygon::new(LineString::new(vec![start, end, start]), vec![]);

        assert_eq!(poly.interior_point(), Some(start.into()));
    }
    #[test]
    fn polygon_vertex_on_median() {
        let poly = Polygon::new(
            LineString::from(vec![
                (0.5, 1.0),
                (0.5, 0.5),
                (0.0, 0.5),
                (0.0, 0.0),
                (1.0, 0.0),
                (1.0, 1.0),
                (0.5, 1.0),
            ]),
            vec![],
        );
        let interior_point = poly.interior_point().unwrap();
        assert_eq!(&interior_point, &p(0.75, 0.75));
    }
    #[test]
    fn multi_poly_with_flat_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![p(0., 0.), p(1., 0.), p(0., 0.)]),
            vec![],
        );
        let multipoly = MultiPolygon::new(vec![poly]);
        assert_eq!(multipoly.interior_point(), Some(p(0.5, 0.)));
    }
    #[test]
    fn multi_poly_with_multiple_flat_polygon_test() {
        let p1 = Polygon::new(
            LineString::from(vec![p(1., 1.), p(1., 3.), p(1., 1.)]),
            vec![],
        );
        let p2 = Polygon::new(
            LineString::from(vec![p(2., 2.), p(6., 2.), p(2., 2.)]),
            vec![],
        );
        let multipoly = MultiPolygon::new(vec![p1, p2]);
        let interior = multipoly.interior_point().unwrap();
        assert_eq!(&interior, &p(1., 2.));
        assert!(multipoly.intersects(&interior));
    }
    #[test]
    fn multi_poly_with_only_points_test() {
        let p1 = Polygon::new(
            LineString::from(vec![p(1., 1.), p(1., 1.), p(1., 1.)]),
            vec![],
        );
        assert_eq!(p1.interior_point(), Some(p(1., 1.)));
        let p2 = Polygon::new(
            LineString::from(vec![p(2., 2.), p(2., 2.), p(2., 2.)]),
            vec![],
        );
        let multipoly = MultiPolygon::new(vec![p1, p2]);
        let interior_point = multipoly.interior_point().unwrap();
        assert_eq!(multipoly.interior_point(), Some(p(1.0, 1.0)));
        assert!(multipoly.intersects(&interior_point));
    }
    #[test]
    fn multi_poly_with_one_ring_and_one_real_poly() {
        // 如果多多边形是由一个“普通”多边形（面积不为零）和一个环（面积为零的多边形）组成，那么多多边形的内部点就是“普通”多边形的内部点。
        let normal = Polygon::new(
            LineString::from(vec![p(1., 1.), p(1., 3.), p(3., 1.), p(1., 1.)]),
            vec![],
        );
        let flat = Polygon::new(
            LineString::from(vec![p(2., 2.), p(6., 2.), p(2., 2.)]),
            vec![],
        );
        let multipoly = MultiPolygon::new(vec![normal.clone(), flat]);
        assert_eq!(multipoly.interior_point(), normal.interior_point());
    }
    #[test]
    fn polygon_flat_interior_test() {
        let poly = Polygon::new(
            LineString::from(vec![p(0., 0.), p(0., 1.), p(1., 1.), p(1., 0.), p(0., 0.)]),
            vec![LineString::from(vec![
                p(0.1, 0.1),
                p(0.1, 0.9),
                p(0.1, 0.1),
            ])],
        );
        assert_eq!(poly.interior_point(), Some(p(0.55, 0.5)));
    }
    #[test]
    fn empty_interior_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![p(0., 0.), p(0., 1.), p(1., 1.), p(1., 0.), p(0., 0.)]),
            vec![LineString::new(vec![])],
        );
        assert_eq!(poly.interior_point(), Some(p(0.5, 0.5)));
    }
    #[test]
    fn polygon_ring_test() {
        let square = LineString::from(vec![p(0., 0.), p(0., 1.), p(1., 1.), p(1., 0.), p(0., 0.)]);
        let poly = Polygon::new(square.clone(), vec![square]);
        let interior_point = poly.interior_point().unwrap();
        assert_eq!(&interior_point, &p(0.0, 0.5));
        assert!(poly.intersects(&interior_point));
        assert!(!poly.contains(&interior_point)); // 没有内部，因此不属于“包含”
    }
    #[test]
    fn polygon_cell_test() {
        // 测试面积为零的多边形的interior_point
        // 这是一个有2个内多边形的多边形，它使外部成为一个分区。
        let square = LineString::from(vec![p(0., 0.), p(0., 2.), p(2., 2.), p(2., 0.), p(0., 0.)]);
        let bottom = LineString::from(vec![p(0., 0.), p(2., 0.), p(2., 1.), p(0., 1.), p(0., 0.)]);
        let top = LineString::from(vec![p(0., 1.), p(2., 1.), p(2., 2.), p(0., 2.), p(0., 1.)]);
        let poly = Polygon::new(square, vec![top, bottom]);
        let interior_point = poly.interior_point().unwrap();
        assert!(poly.intersects(&interior_point));
        assert!(!poly.contains(&interior_point));
    }
    // 测试：MultiPolygon的内部点
    #[test]
    fn empty_multipolygon_polygon_test() {
        assert!(MultiPolygon::<f64>::new(Vec::new())
            .interior_point()
            .is_none());
    }

    #[test]
    fn multipolygon_one_polygon_test() {
        let linestring =
            LineString::from(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert_eq!(
            MultiPolygon::new(vec![poly]).interior_point(),
            Some(p(1., 1.))
        );
    }
    #[test]
    fn multipolygon_two_polygons_test() {
        let linestring =
            LineString::from(vec![p(2., 1.), p(5., 1.), p(5., 3.), p(2., 3.), p(2., 1.)]);
        let poly1 = Polygon::new(linestring, Vec::new());
        let linestring =
            LineString::from(vec![p(7., 1.), p(8., 1.), p(8., 2.), p(7., 2.), p(7., 1.)]);
        let poly2 = Polygon::new(linestring, Vec::new());
        let multipoly = MultiPolygon::new(vec![poly1, poly2]);
        let interior_point = multipoly.interior_point().unwrap();
        assert_relative_eq!(interior_point, point![x: 3.5, y: 2.]);
        assert!(multipoly.contains(&interior_point));
    }
    #[test]
    fn multipolygon_two_polygons_of_opposite_clockwise_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly1 = Polygon::new(linestring, Vec::new());
        let linestring = LineString::from(vec![(0., 0.), (-2., 0.), (-2., 2.), (0., 2.), (0., 0.)]);
        let poly2 = Polygon::new(linestring, Vec::new());
        let multipoly = MultiPolygon::new(vec![poly1, poly2]);
        let interior_point = multipoly.interior_point().unwrap();
        assert_relative_eq!(interior_point, point![x: 1.0, y: 1.0]);
        assert!(multipoly.contains(&interior_point));
    }
    #[test]
    fn bounding_rect_test() {
        let bounding_rect = Rect::new(coord! { x: 0., y: 50. }, coord! { x: 4., y: 100. });
        let point = point![x: 2., y: 75.];
        assert_eq!(point, bounding_rect.interior_point());
    }
    #[test]
    fn line_test() {
        let line1 = Line::new(c(0., 1.), c(1., 3.));
        assert_eq!(line1.interior_point(), point![x: 0., y: 1.]);
    }
    #[test]
    fn collection_test() {
        let p0 = point!(x: 0.0, y: 0.0);
        let p1 = point!(x: 2.0, y: 0.0);
        let p2 = point!(x: 2.0, y: 2.0);
        let p3 = point!(x: 0.0, y: 2.0);

        let multi_point = MultiPoint::new(vec![p0, p1, p2, p3]);
        assert_eq!(
            multi_point.interior_point().unwrap(),
            point!(x: 0.0, y: 0.0)
        );
    }
    #[test]
    fn mixed_collection_test() {
        let linestring =
            LineString::from(vec![p(0., 1.), p(0., 0.), p(1., 0.), p(1., 1.), p(0., 1.)]);
        let poly1 = Polygon::new(linestring, Vec::new());
        let linestring = LineString::from(vec![
            p(10., 1.),
            p(10., 0.),
            p(11., 0.),
            p(11., 1.),
            p(10., 1.),
        ]);
        let poly2 = Polygon::new(linestring, Vec::new());

        let high_dimension_shapes = GeometryCollection::new_from(vec![poly1.into(), poly2.into()]);

        let mut mixed_shapes = high_dimension_shapes.clone();
        mixed_shapes.0.push(Point::new(5_f64, 0_f64).into());
        mixed_shapes.0.push(Point::new(5_f64, 1_f64).into());

        // 如果存在较高维度的形状，较低维度的形状不应影响内部点，即使低维形状更接近质心。
        assert_eq!(
            high_dimension_shapes.interior_point().unwrap(),
            mixed_shapes.interior_point().unwrap()
        )
    }
    #[test]
    fn triangles() {
        // 普通三角形
        assert_eq!(
            Triangle::new(c(0., 0.), c(3., 0.), c(1.5, 3.)).interior_point(),
            point!(x: 1.5, y: 1.0)
        );

        // 平坦三角形
        assert_eq!(
            Triangle::new(c(0., 0.), c(3., 0.), c(1., 0.)).interior_point(),
            point!(x: 1.5, y: 0.0)
        );

        // 非轴对齐的平坦三角形
        assert_eq!(
            Triangle::new(c(0., 0.), c(3., 3.), c(1., 1.)).interior_point(),
            point!(x: 1.5, y: 1.5)
        );

        // 带有一些重复点的三角形
        assert_eq!(
            Triangle::new(c(0., 0.), c(0., 0.), c(1., 0.)).interior_point(),
            point!(x: 0.5, y: 0.0)
        );

        // 所有点重复的三角形
        assert_eq!(
            Triangle::new(c(0., 0.5), c(0., 0.5), c(0., 0.5)).interior_point(),
            point!(x: 0., y: 0.5)
        )
    }

    #[test]
    fn degenerate_triangle_like_ring() {
        let triangle = Triangle::new(c(0., 0.), c(1., 1.), c(2., 2.));
        let poly: Polygon<_> = triangle.into();

        let line = Line::new(c(0., 1.), c(1., 3.));

        let g1 = GeometryCollection::new_from(vec![triangle.into(), line.into()]);
        let g2 = GeometryCollection::new_from(vec![poly.into(), line.into()]);

        let pt1 = g1.interior_point().unwrap();
        let pt2 = g2.interior_point().unwrap();
        // 三角形和多边形具有不同的内部点实现，因此我们不会在这两种方法中获得相同的点，但都应该产生点，这些点是任何一种表示的内部点
        assert!(g1.intersects(&pt1));
        assert!(g1.intersects(&pt2));
        assert!(g2.intersects(&pt1));
        assert!(g2.intersects(&pt2));
    }

    #[test]
    fn degenerate_rect_like_ring() {
        let rect = Rect::new(c(0., 0.), c(0., 4.));
        let poly: Polygon<_> = rect.into();

        let line = Line::new(c(0., 1.), c(1., 3.));

        let g1 = GeometryCollection::new_from(vec![rect.into(), line.into()]);
        let g2 = GeometryCollection::new_from(vec![poly.into(), line.into()]);
        assert_eq!(g1.interior_point(), g2.interior_point());
    }

    #[test]
    fn rectangles() {
        // 普通矩形
        assert_eq!(
            Rect::new(c(0., 0.), c(4., 4.)).interior_point(),
            point!(x: 2.0, y: 2.0)
        );

        //平坦矩形
        assert_eq!(
            Rect::new(c(0., 0.), c(4., 0.)).interior_point(),
            point!(x: 2.0, y: 0.0)
        );

        //所有点重复的矩形
        assert_eq!(
            Rect::new(c(4., 4.), c(4., 4.)).interior_point(),
            point!(x: 4., y: 4.)
        );

        // 含有矩形的集合
        let collection = GeometryCollection::new_from(vec![
            p(0., 0.).into(),
            p(6., 0.).into(),
            p(6., 6.).into(),
        ]);
        // 检查集合
        assert_eq!(collection.interior_point().unwrap(), point!(x: 6., y: 0.));
    }
}
