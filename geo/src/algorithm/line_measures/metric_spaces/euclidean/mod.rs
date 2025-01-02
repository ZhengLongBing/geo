mod distance;

use super::super::{Distance, InterpolatePoint};
use crate::line_measures::densify::densify_between;
use crate::{CoordFloat, Point};
use num_traits::FromPrimitive;

/// 在[欧几里得平面]上的操作使用毕达哥拉斯公式计算距离 - 就像你用尺子测量一样。
///
/// 如果你有经/纬度坐标点，使用[`Haversine`]、[`Geodesic`]或其他[度量空间] - 欧几里得方法将会产生不合逻辑的结果。
///
/// 如果你希望在经/纬度数据上使用欧几里得操作，必须先使用[`Transform::transform`](crate::Transform::transform) / [`Transform::transform_crs_to_crs`](crate::Transform::transform_crs_to_crs)方法或它们的不可变版本对坐标进行转换。这需要使用 `proj` 功能。
///
/// [欧几里得平面]: https://en.wikipedia.org/wiki/Euclidean_plane
/// [`Transform`]: crate::Transform
/// [`Haversine`]: super::Haversine
/// [`Geodesic`]: super::Geodesic
/// [度量空间]: super
pub struct Euclidean;

/// 在[欧几里得平面]上沿线插值点。
///
/// [欧几里得平面]: https://en.wikipedia.org/wiki/Euclidean_plane
impl<F: CoordFloat + FromPrimitive> InterpolatePoint<F> for Euclidean {
    /// 返回沿着从`start`到`end`线段的指定距离上的点。
    ///
    /// # 单位
    /// - `distance`: 使用`start`和`end`点的单位进行测量。
    ///
    ///   `distance`以及`start`和`end`点应该使用非角度单位，比如米或英里，而不是经/纬度。
    ///   对于经/纬度点，使用[`Haversine`]或[`Geodesic`] [度量空间]。
    ///
    /// [`Haversine`]: crate::line_measures::Haversine
    /// [`Geodesic`]: crate::line_measures::Geodesic
    /// [度量空间]: crate::line_measures::metric_spaces
    fn point_at_distance_between(
        start: Point<F>,
        end: Point<F>,
        distance_from_start: F,
    ) -> Point<F> {
        let diff = end - start;
        let total_distance = diff.x().hypot(diff.y());
        let offset = diff * distance_from_start / total_distance;
        start + offset
    }

    /// 返回沿着从`start`到`end`线段的指定比例上的点。
    ///
    /// # 单位
    /// - `distance`: 使用`start`和`end`点的单位进行测量。
    ///
    ///   `distance`以及`start`和`end`点应该使用非角度单位，比如米或英里，而不是经/纬度。
    ///   对于经/纬度点，使用[`Haversine`]或[`Geodesic`] [度量空间]。
    ///
    /// [`Haversine`]: crate::line_measures::Haversine
    /// [`Geodesic`]: crate::line_measures::Geodesic
    /// [度量空间]: crate::line_measures::metric_spaces
    fn point_at_ratio_between(start: Point<F>, end: Point<F>, ratio_from_start: F) -> Point<F> {
        let diff = end - start;
        start + diff * ratio_from_start
    }

    /// 在`start`和`end`之间插值`Point`。
    ///
    /// 将添加尽可能多的点，以使两点之间的距离从不超过`max_distance`。如果起点和终点之间的距离小于`max_distance`，则输出中不会包含其他点。
    ///
    /// `include_ends`: 是否应在输出中包含起点和终点？
    ///
    /// # 单位
    /// - `max_distance`: 使用`start`和`end`点的单位进行测量。
    ///
    ///   `max_distance`以及`start`和`end`点应该使用非角度单位，比如米或英里，而不是经/纬度。
    ///   对于经/纬度点，使用[`Haversine`]或[`Geodesic`] [度量空间]。
    ///
    /// [`Haversine`]: crate::line_measures::Haversine
    /// [`Geodesic`]: crate::line_measures::Geodesic
    /// [度量空间]: crate::line_measures::metric_spaces
    fn points_along_line(
        start: Point<F>,
        end: Point<F>,
        max_distance: F,
        include_ends: bool,
    ) -> impl Iterator<Item = Point<F>> {
        let mut container = vec![];
        if include_ends {
            container.push(start);
        }
        densify_between::<F, Self>(start, end, &mut container, max_distance);
        if include_ends {
            container.push(end);
        }
        container.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type MetricSpace = Euclidean;

    mod distance {
        use super::*;

        #[test]
        fn new_york_to_london() {
            // web 墨卡托
            let new_york_city = Point::new(-8238310.24, 4942194.78);
            // web 墨卡托
            let london = Point::new(-14226.63, 6678077.70);
            let distance: f64 = MetricSpace::distance(new_york_city, london);

            assert_relative_eq!(
                8_405_286., // web 墨卡托的米
                distance.round()
            );
        }

        #[test]
        fn test_point_at_distance_between() {
            let new_york_city = Point::new(-8_238_310.24, 4_942_194.78);
            // web 墨卡托
            let london = Point::new(-14_226.63, 6_678_077.70);
            let start = MetricSpace::point_at_distance_between(new_york_city, london, 0.0);
            assert_relative_eq!(new_york_city, start);

            let midway =
                MetricSpace::point_at_distance_between(new_york_city, london, 8_405_286.0 / 2.0);
            assert_relative_eq!(Point::new(-4_126_268., 5_810_136.), midway, epsilon = 1.0);

            let end = MetricSpace::point_at_distance_between(new_york_city, london, 8_405_286.0);
            assert_relative_eq!(london, end, epsilon = 1.0);
        }
    }
}
