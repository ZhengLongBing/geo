use super::{Distance, InterpolatePoint};
use crate::{
    CoordFloat, CoordsIter, Line, LineString, MultiLineString, MultiPolygon, Point, Polygon, Rect,
    Triangle,
};
use num_traits::FromPrimitive;

/// 创建一个几何图形的副本，根据需要插入附加点，以确保点与点之间的距离不超过 `max_segment_length`。
///
/// ## 单位
/// - `max_segment_length` 的单位取决于实现的[度量空间]。它必须大于0。
///
/// # 示例
/// ```
/// # use approx::assert_relative_eq;
/// use geo::{wkt, Densify};
/// use geo::line_measures::Euclidean;
///
/// let line_string = wkt!(LINESTRING(0.0 0.0,0.0 6.0,1.0 7.0));
///
/// // 对于欧几里得计算，距离的单位与坐标的单位相同。
/// let max_dist = 2.0;
/// let densified = line_string.densify::<Euclidean>(max_dist);
/// let expected_output = wkt!(LINESTRING(
///     0.0 0.0,
///     0.0 2.0,
///     0.0 4.0,
///     0.0 6.0,
///     1.0 7.0
/// ));
/// assert_relative_eq!(densified, expected_output);
///```
///
/// 对于经度/纬度几何图形，考虑使用其他[度量空间]，如 [`Haversine`](crate::Haversine) 或 [`Geodesic`](crate::Geodesic)。
///
/// ```
/// # use approx::assert_relative_eq;
/// use geo::{wkt, Densify};
/// use geo::line_measures::Haversine;
/// let line_string = wkt!(LINESTRING(0.0 0.0,0.0 6.0,1.0 7.0));
///
/// // 对于Haversine，距离单位是米
/// let max_dist = 200_000.0;
/// let densified = line_string.densify::<Haversine>(max_dist);
/// // Haversine 将坐标点解释为经度/纬度
/// let expected_output = wkt!(LINESTRING(
///     0.0 0.0,
///     0.0 1.5,
///     0.0 3.0,
///     0.0 4.5,
///     0.0 6.0,
///     1.0 7.0
/// ));
/// assert_relative_eq!(densified, expected_output, epsilon = 1e-14);
/// ```
/// [度量空间]: crate::line_measures::metric_spaces
pub trait Densify<F: CoordFloat> {
    type Output;
    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>;
}

pub(crate) fn densify_between<F, MetricSpace>(
    line_start: Point<F>,
    line_end: Point<F>,
    container: &mut Vec<Point<F>>,
    max_segment_length: F,
) where
    F: CoordFloat + FromPrimitive,
    MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
{
    assert!(max_segment_length > F::zero());
    let num_segments = (MetricSpace::distance(line_start, line_end) / max_segment_length)
        .ceil()
        .to_u64()
        .expect("段数不合理");

    // 此线段的距离“单位”
    let frac = F::one() / F::from(num_segments).unwrap();

    for segment_num in 1..num_segments {
        let ratio = frac * F::from(segment_num).unwrap();

        // 性能TODO：我们在循环的每一步都重新计算“total_distance”。
        // 如果我们实现 point_at_distance_between，我们可以计算一次然后在这里使用。
        // 就此而言，我认为这个函数可能是为所有度量空间统一通用points_along_line的*唯一*基础。
        let interpolated_point = MetricSpace::point_at_ratio_between(line_start, line_end, ratio);
        container.push(interpolated_point);
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for Line<F> {
    type Output = LineString<F>;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        let mut points = vec![self.start_point()];
        densify_between::<F, MetricSpace>(
            self.start_point(),
            self.end_point(),
            &mut points,
            max_segment_length,
        );
        points.push(self.end_point());
        LineString::from(points)
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for LineString<F> {
    type Output = Self;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> LineString<F>
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        if self.coords_count() == 0 {
            return LineString::new(vec![]);
        }

        let mut points = vec![];
        self.lines().for_each(|line| {
            points.push(line.start_point());
            densify_between::<F, MetricSpace>(
                line.start_point(),
                line.end_point(),
                &mut points,
                max_segment_length,
            )
        });

        // 完成后，推入最后一个坐标以结束
        let final_coord = *self.0.last().expect("我们已经断言线字符串不为空");
        points.push(final_coord.into());

        LineString::from(points)
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for MultiLineString<F> {
    type Output = Self;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        MultiLineString::new(
            self.iter()
                .map(|line_string| line_string.densify::<MetricSpace>(max_segment_length))
                .collect(),
        )
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for Polygon<F> {
    type Output = Self;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        Polygon::new(
            self.exterior().densify::<MetricSpace>(max_segment_length),
            self.interiors()
                .iter()
                .map(|interior| interior.densify::<MetricSpace>(max_segment_length))
                .collect(),
        )
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for MultiPolygon<F> {
    type Output = Self;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        MultiPolygon::new(
            self.iter()
                .map(|polygon| polygon.densify::<MetricSpace>(max_segment_length))
                .collect(),
        )
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for Rect<F> {
    type Output = Polygon<F>;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        self.to_polygon().densify::<MetricSpace>(max_segment_length)
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for Triangle<F> {
    type Output = Polygon<F>;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        self.to_polygon().densify::<MetricSpace>(max_segment_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord, polygon, wkt, Euclidean, Geodesic, Haversine, Rhumb};

    #[test]
    fn densify_line() {
        // 伦敦到巴黎
        let line = Line::new(
            coord!(x: -0.1278f64, y: 51.5074),
            coord!(x: 2.3522, y: 48.8566),
        );

        let densified_line = line.densify::<Geodesic>(100_000.0); // 最大线段长度100km
        assert!(densified_line.coords_count() > 2);

        let densified_rhumb = line.densify::<Rhumb>(100_000.0);
        assert!(densified_rhumb.coords_count() > 2);

        let densified_haversine = line.densify::<Haversine>(100_000.0);
        assert!(densified_haversine.coords_count() > 2);
    }

    #[test]
    fn densify_line_string() {
        let line_string = LineString::new(vec![
            coord!(x: -58.3816f64, y: -34.6037), // 布宜诺斯艾利斯, 阿根廷
            coord!(x: -77.0428, y: -12.0464),    // 利马, 秘鲁
            coord!(x: -47.9292, y: -15.7801),    // 巴西利亚, 巴西
        ]);

        let densified_ls = line_string.densify::<Geodesic>(500_000.0); // 最大线段长度500公里
        assert!(densified_ls.coords_count() > line_string.coords_count());

        let densified_rhumb_ls = line_string.densify::<Rhumb>(500_000.0);
        assert!(densified_rhumb_ls.coords_count() > line_string.coords_count());

        let densified_haversine_ls = line_string.densify::<Haversine>(500_000.0);
        assert!(densified_haversine_ls.coords_count() > line_string.coords_count());
    }

    #[test]
    fn densify_polygon() {
        let polygon = polygon![
            (x: -58.3816f64, y: -34.6037), // 布宜诺斯艾利斯
            (x: -77.0428, y: -12.0464),    // 利马
            (x: -47.9292, y: -15.7801),    // 巴西利亚
        ];

        let densified_polygon = polygon.densify::<Geodesic>(500_000.0); // 最大线段长度500公里
        assert!(densified_polygon.exterior().coords_count() > polygon.exterior().coords_count());
    }

    // 从旧的已弃用trait移植，仅适用于欧几里得度量
    mod euclidean {
        use super::*;

        #[test]
        fn test_polygon_densify() {
            let polygon = wkt!(POLYGON(
                (-5.0 0.0,0.0 5.0,5.0 0.0,-5.0 0.0),
                (-3.0 0.0,0.0 3.0,3.0 0.0,-3.0 0.0)
            ));

            let expected = wkt!(POLYGON(
                (-5.0 0.0,-3.75 1.25,-2.5 2.5,-1.25 3.75,0.0 5.0,1.25 3.75,2.5 2.5,3.75 1.25,5.0 0.0,3.0 0.0,1.0 0.0,-1.0000000000000009 0.0,-3.0 0.0, -5.0 0.0),
                (-3.0 0.0,-2.0 1.0,-1.0 2.0,0.0 3.0,1.0 2.0,2.0 1.0,3.0 0.0,1.0 0.0,-1.0 0.0,-3.0 0.0)
            ));

            let max_dist = 2.0;
            let densified = polygon.densify::<Euclidean>(max_dist);
            assert_eq!(densified, expected);
        }

        #[test]
        fn test_empty_linestring_densify() {
            let linestring = LineString::<f64>::new(vec![]);
            let max_dist = 2.0;
            let densified = linestring.densify::<Euclidean>(max_dist);
            assert!(densified.0.is_empty());
        }

        #[test]
        fn test_linestring_densify() {
            let linestring = wkt!(LINESTRING(
               -1.0 0.0,
                0.0 0.0,
                0.0 6.0,
                1.0 8.0
            ));
            let expected = wkt!(LINESTRING(
               -1.0 0.0,
                0.0 0.0,
                0.0 2.0,
                0.0 4.0,
                0.0 6.0,
                0.5 7.0,
                1.0 8.0
            ));
            let max_dist = 2.0;
            let densified = linestring.densify::<Euclidean>(max_dist);
            assert_eq!(densified, expected);
        }

        #[test]
        fn test_line_densify() {
            let line: Line<f64> = Line::new(coord! {x: 0.0, y: 6.0}, coord! {x: 1.0, y: 8.0});
            let correct: LineString<f64> = vec![[0.0, 6.0], [0.5, 7.0], [1.0, 8.0]].into();
            let max_dist = 2.0;
            let densified = line.densify::<Euclidean>(max_dist);
            assert_eq!(densified, correct);
        }
    }

    // 从现在废弃的DensifyHaversine移植
    mod lon_lat_tests {
        use super::*;

        #[test]
        fn test_polygon_densify() {
            let polygon = wkt!(POLYGON((
                4.925 45.804,
                4.732 45.941,
                4.935 46.513,
                5.821 46.103,
                5.627 45.611,
                5.355 45.883,
                4.925 45.804
            )));

            let exepcted_haversine = wkt!(POLYGON((
                4.925 45.804,
                4.732 45.941,
                4.8329711649985505 46.2270449096239,
                4.935 46.513,
                5.379659133344039 46.30885540136222,
                5.821 46.103,
                5.723570877658867 45.85704103535437,
                5.627 45.611,
                5.355 45.883,
                4.925 45.804
            )));

            let actual_haversine = polygon.densify::<Haversine>(50000.0);
            assert_relative_eq!(actual_haversine, exepcted_haversine);

            let expected_geodesic = wkt!(POLYGON((
                4.925 45.804,
                4.732 45.941,
                4.832972865149862 46.22705224065524,
                4.935 46.513,
                5.379653814979939 46.30886184400083,
                5.821 46.103,
                5.723572275808633 45.85704648840237,
                5.627 45.611,
                5.355 45.883,
                4.925 45.804
            )));
            let actual_geodesic = polygon.densify::<Geodesic>(50000.0);
            assert_relative_eq!(actual_geodesic, expected_geodesic);
        }

        #[test]
        fn test_linestring_densify() {
            let linestring = wkt!(LINESTRING(
                -3.202 55.9471,
                -3.2012 55.9476,
                -3.1994 55.9476,
                -3.1977 55.9481,
                -3.196 55.9483,
                -3.1947 55.9487,
                -3.1944 55.9488,
                -3.1944 55.949
            ));

            let expected = wkt!(LINESTRING(
                -3.202 55.9471,
                -3.2012 55.9476,
                -3.2002999999999995 55.94760000327935,
                -3.1994 55.9476,
                -3.1985500054877773 55.94785000292509,
                -3.1977 55.9481,
                -3.196 55.9483,
                -3.1947 55.9487,
                -3.1944 55.9488,
                -3.1944 55.949
            ));

            let dense = linestring.densify::<Haversine>(110.0);
            assert_relative_eq!(dense, expected);
        }

        #[test]
        fn test_line_densify() {
            let output = wkt!(LINESTRING(0.0 0.0, 0.0 0.5, 0.0 1.0));
            let line = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 0.0, y: 1.0 });
            let dense = line.densify::<Haversine>(100000.0);
            assert_relative_eq!(dense, output);
        }
    }

    mod degenerate {
        use super::*;

        #[test]
        fn test_empty_linestring() {
            let input = wkt!(LINESTRING EMPTY);
            let dense = input.densify::<Euclidean>(1.0);
            assert_eq!(0, dense.coords_count());
            assert_eq!(input, dense);
        }

        #[test]
        fn test_one_point_linestring() {
            let input = wkt!(LINESTRING(1.0 1.0));
            let dense = input.densify::<Euclidean>(1.0);
            assert_eq!(1, dense.coords_count());
            assert_eq!(input, dense);
        }
    }
}
