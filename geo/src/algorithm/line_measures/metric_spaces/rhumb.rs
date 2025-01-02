use num_traits::FromPrimitive;

use super::super::{Bearing, Destination, Distance, InterpolatePoint};
use crate::rhumb::RhumbCalculations;
use crate::{CoordFloat, Point, MEAN_EARTH_RADIUS};

/// 提供 [rhumb line] （亦称为等航线）几何操作。在墨卡托投影地图上，等航线呈现为直线。
///
/// 等航线距离以米为单位测量。
///
/// # 参考资料
///
/// 距离、目的地和方位的实现部分借鉴自 [Turf.js](https://turfjs.org/) 中的等效实现，
/// 这又源于 Movable Type 的 [spherical geodesy tools](https://www.movable-type.co.uk/scripts/latlong.html)。
///
/// Turf.js 版权归其作者所有，地理工具版权归 Chris Veness 所有；两者均可在MIT许可协议下使用。
///
/// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
pub struct Rhumb;

impl<F: CoordFloat + FromPrimitive> Bearing<F> for Rhumb {
    /// 返回从 `origin` 到 `destination` 沿 [rhumb line] 的方位，以度为单位。
    ///
    /// # 单位
    ///
    /// - `origin`，`destination`：x/y 为经纬度坐标的点
    /// - 返回：度（北：0°，东：90°，南：180°，西：270°）
    ///
    /// # 例子
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Rhumb, Bearing};
    /// use geo::Point;
    ///
    /// let origin = Point::new(9.177789688110352, 48.776781529534965);
    /// let destination = Point::new(9.274348757829898, 48.84037308229984);
    /// let bearing = Rhumb::bearing(origin, destination);
    /// assert_relative_eq!(bearing, 45., epsilon = 1.0e-6);
    /// ```
    ///
    /// # 参考资料
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    ///
    /// Bullock, R.: Great Circle Distances and Bearings Between Two Locations, 2007.
    /// (<https://dtcenter.org/met/users/docs/write_ups/gc_simple.pdf>)
    fn bearing(origin: Point<F>, destination: Point<F>) -> F {
        let three_sixty = F::from(360.0f64).unwrap();

        let calculations = RhumbCalculations::new(&origin, &destination);
        (calculations.theta().to_degrees() + three_sixty) % three_sixty
    }
}

impl<F: CoordFloat + FromPrimitive> Destination<F> for Rhumb {
    /// 返回一个新点，该点是从 `origin` 点沿 [rhumb line] 以给定的 `bearing` 行驶了 `distance` 的结果。
    ///
    /// # 单位
    ///
    /// - `origin`：x/y 为经纬度坐标的点
    /// - `bearing`：度（北：0°，东：90°，南：180°，西：270°）
    /// - `distance`：米
    /// - 返回：x/y 为经纬度坐标的点
    ///
    /// # 例子
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Rhumb, Destination};
    /// use geo::Point;
    ///
    /// let p_1 = Point::new(9.177789688110352, 48.776781529534965);
    /// let p_2 = Rhumb::destination(p_1, 45., 10000.);
    /// assert_relative_eq!(p_2, Point::new(9.274348757829898, 48.84037308229984))
    /// ```
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn destination(origin: Point<F>, bearing: F, distance: F) -> Point<F> {
        let delta = distance / F::from(MEAN_EARTH_RADIUS).unwrap(); // 以弧度为单位的角距离
        let lambda1 = origin.x().to_radians();
        let phi1 = origin.y().to_radians();
        let theta = bearing.to_radians();

        crate::algorithm::rhumb::calculate_destination(delta, lambda1, phi1, theta)
    }
}

impl<F: CoordFloat + FromPrimitive> Distance<F, Point<F>, Point<F>> for Rhumb {
    /// 确定两个点之间沿 [rhumb line] 的距离。
    ///
    /// # 单位
    ///
    /// - `origin`，`destination`：x/y 为经纬度坐标的点
    /// - 返回：米
    ///
    /// # 例子
    ///
    /// ```
    /// use geo::{Rhumb, Distance};
    /// use geo::point;
    ///
    /// // 纽约市
    /// let p1 = point!(x: -74.006f64, y: 40.7128);
    ///
    /// // 伦敦
    /// let p2 = point!(x: -0.1278, y: 51.5074);
    ///
    /// let distance = Rhumb::distance(p1, p2);
    ///
    /// assert_eq!(
    ///     5_794_129., // 米
    ///     distance.round()
    /// );
    /// ```
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn distance(origin: Point<F>, destination: Point<F>) -> F {
        let calculations = RhumbCalculations::new(&origin, &destination);
        calculations.delta() * F::from(MEAN_EARTH_RADIUS).unwrap()
    }
}

/// 沿着 [rhumb line] 插入点。
///
/// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
impl<F: CoordFloat + FromPrimitive> InterpolatePoint<F> for Rhumb {
    /// 返回一个新点，该点位于两个现有点之间的 [rhumb line] 上。
    ///
    /// # 例子
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Rhumb, InterpolatePoint};
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    ///
    /// let closer_to_p1 = Rhumb::point_at_distance_between(p1, p2, 100_000.0);
    /// assert_relative_eq!(closer_to_p1, Point::new(10.96, 20.04), epsilon = 1.0e-2);
    ///
    /// let closer_to_p2 = Rhumb::point_at_distance_between(p1, p2, 10_000_000.0);
    /// assert_relative_eq!(closer_to_p2, Point::new(107.00, 24.23), epsilon = 1.0e-2);
    /// ```
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn point_at_distance_between(start: Point<F>, end: Point<F>, meters_from_start: F) -> Point<F> {
        let bearing = Self::bearing(start, end);
        Self::destination(start, bearing, meters_from_start)
    }

    /// 返回一个新点，该点位于两个现有点之间的 [rhumb line] 上。
    ///
    /// # 例子
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Rhumb, InterpolatePoint};
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    ///
    /// let closer_to_p1 = Rhumb::point_at_ratio_between(p1, p2, 0.1);
    /// assert_relative_eq!(closer_to_p1, Point::new(21.32, 20.50), epsilon = 1.0e-2);
    ///
    /// let closer_to_p2 = Rhumb::point_at_ratio_between(p1, p2, 0.9);
    /// assert_relative_eq!(closer_to_p2, Point::new(113.31, 24.50), epsilon = 1.0e-2);
    ///
    /// let midpoint = Rhumb::point_at_ratio_between(p1, p2, 0.5);
    /// assert_relative_eq!(midpoint, Point::new(66.98, 22.50), epsilon = 1.0e-2);
    /// ```
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn point_at_ratio_between(start: Point<F>, end: Point<F>, ratio_from_start: F) -> Point<F> {
        let calculations = RhumbCalculations::new(&start, &end);
        calculations.intermediate(ratio_from_start)
    }

    /// 在 `start` 和 `end` 之间的 [rhumb line] 上插入 `Point`。
    ///
    /// 将根据需要添加多个点，以确保点之间的距离不超过 `max_distance`。
    /// 如果起点和终点之间的距离小于 `max_distance`，则输出中不再包含额外的点。
    ///
    /// `include_ends`：是否应在输出中包含起始点和终点？
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn points_along_line(
        start: Point<F>,
        end: Point<F>,
        max_distance: F,
        include_ends: bool,
    ) -> impl Iterator<Item = Point<F>> {
        let max_delta = max_distance / F::from(MEAN_EARTH_RADIUS).unwrap();
        let calculations = RhumbCalculations::new(&start, &end);
        calculations
            .intermediate_fill(max_delta, include_ends)
            .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type MetricSpace = Rhumb;

    mod bearing {
        use super::*;

        #[test]
        fn north() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(0.0, 1.0);
            assert_relative_eq!(0.0, MetricSpace::bearing(origin, destination));
        }

        #[test]
        fn east() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(1.0, 0.0);
            assert_relative_eq!(90.0, MetricSpace::bearing(origin, destination));
        }

        #[test]
        fn south() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(0.0, -1.0);
            assert_relative_eq!(180.0, MetricSpace::bearing(origin, destination));
        }

        #[test]
        fn west() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(-1.0, 0.0);
            assert_relative_eq!(270.0, MetricSpace::bearing(origin, destination));
        }
    }

    mod destination {
        use super::*;

        #[test]
        fn north() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 0.0;
            assert_relative_eq!(
                Point::new(0.0, 0.899320363724538),
                MetricSpace::destination(origin, bearing, 100_000.0)
            );
        }

        #[test]
        fn east() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 90.0;
            assert_relative_eq!(
                Point::new(0.8993203637245415, 5.506522912913066e-17),
                MetricSpace::destination(origin, bearing, 100_000.0)
            );
        }

        #[test]
        fn south() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 180.0;
            assert_relative_eq!(
                Point::new(0.0, -0.899320363724538),
                MetricSpace::destination(origin, bearing, 100_000.0)
            );
        }

        #[test]
        fn west() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 270.0;
            assert_relative_eq!(
                Point::new(-0.8993203637245415, -1.6520247072649334e-16),
                MetricSpace::destination(origin, bearing, 100_000.0)
            );
        }
    }

    mod distance {
        use super::*;

        #[test]
        fn new_york_to_london() {
            let new_york_city = Point::new(-74.006, 40.7128);
            let london = Point::new(-0.1278, 51.5074);

            let distance: f64 = MetricSpace::distance(new_york_city, london);

            assert_relative_eq!(
                5_794_129., // 米
                distance.round()
            );
        }
    }

    mod interpolate_point {
        use super::*;

        #[test]
        fn point_at_ratio_between_midpoint() {
            let start = Point::new(10.0, 20.0);
            let end = Point::new(125.0, 25.0);
            let midpoint = MetricSpace::point_at_ratio_between(start, end, 0.5);
            assert_relative_eq!(
                midpoint,
                Point::new(66.98011173721943, 22.500000000000007),
                epsilon = 1.0e-10
            );
        }
        #[test]
        fn points_along_line_with_endpoints() {
            let start = Point::new(10.0, 20.0);
            let end = Point::new(125.0, 25.0);
            let max_dist = 1000000.0; // 米
            let route =
                MetricSpace::points_along_line(start, end, max_dist, true).collect::<Vec<_>>();
            assert_eq!(route.len(), 13);
            assert_eq!(route[0], start);
            assert_eq!(route.last().unwrap(), &end);
            assert_relative_eq!(
                route[1],
                Point::new(19.43061818495096, 20.416666666666668),
                epsilon = 1.0e-10
            );
        }
        #[test]
        fn points_along_line_without_endpoints() {
            let start = Point::new(10.0, 20.0);
            let end = Point::new(125.0, 25.0);
            let max_dist = 1000000.0; // 米
            let route =
                MetricSpace::points_along_line(start, end, max_dist, false).collect::<Vec<_>>();
            assert_eq!(route.len(), 11);
            assert_relative_eq!(
                route[0],
                Point::new(19.43061818495096, 20.416666666666668),
                epsilon = 1.0e-10
            );
        }
    }
}
