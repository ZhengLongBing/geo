use num_traits::FromPrimitive;

use super::super::{Bearing, Destination, Distance, InterpolatePoint};
use crate::utils::normalize_longitude;
use crate::{CoordFloat, Point, MEAN_EARTH_RADIUS};

/// 使用 [haversine 公式] 的地球球面模型。
///
/// 距离被认为是 [大圆] 长度，并以米为单位测量。
///
/// # 参考文献
///
/// *注意*：该实现使用了基于 [IUGG 推荐](ftp://athena.fsv.cvut.cz/ZFG/grs80-Moritz.pdf) 的 6371.088 km 的平均地球半径。
///
/// [haversine 公式]: https://en.wikipedia.org/wiki/Haversine_formula
/// [大圆]: https://en.wikipedia.org/wiki/Great_circle
pub struct Haversine;

impl<F: CoordFloat + FromPrimitive> Bearing<F> for Haversine {
    /// 返回沿 [大圆] 从 `origin` 到 `destination` 的方位角，以度为单位。
    ///
    /// # 单位
    ///
    /// - `origin`, `destination`: 点，其 x/y 是经纬度坐标
    /// - 返回值：度数，北: 0°, 东: 90°, 南: 180°, 西: 270°
    ///
    /// # 示例
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, Bearing};
    /// use geo::Point;
    ///
    /// let origin = Point::new(9.0, 10.0);
    /// let destination = Point::new(9.5, 10.1);
    /// let bearing = Haversine::bearing(origin, destination);
    /// // 略偏于东
    /// assert_relative_eq!(bearing, 78.47, epsilon = 1.0e-2);
    /// ```
    ///
    /// # 参考文献
    ///
    /// Bullock, R.: Great Circle Distances and Bearings Between Two Locations, 2007.
    /// (<https://dtcenter.org/met/users/docs/write_ups/gc_simple.pdf>)
    ///
    /// [大圆]: https://en.wikipedia.org/wiki/Great_circle
    fn bearing(origin: Point<F>, destination: Point<F>) -> F {
        let three_sixty =
            F::from(360.0).expect("Numeric type to be constructable from primitive 360");
        let (lng_a, lat_a) = (origin.x().to_radians(), origin.y().to_radians());
        let (lng_b, lat_b) = (destination.x().to_radians(), destination.y().to_radians());
        let delta_lng = lng_b - lng_a;
        let s = lat_b.cos() * delta_lng.sin();
        let c = lat_a.cos() * lat_b.sin() - lat_a.sin() * lat_b.cos() * delta_lng.cos();

        let degrees = F::atan2(s, c).to_degrees();
        (degrees + three_sixty) % three_sixty
    }
}

impl<F: CoordFloat + FromPrimitive> Destination<F> for Haversine {
    /// 返回从 `origin` 点沿指定 `bearing` 经过 `distance` 的 [大圆] 新点。
    ///
    /// # 单位
    ///
    /// - `origin`: 点，其 x/y 是经纬度坐标
    /// - `bearing`: 度数，北: 0°, 东: 90°, 南: 180°, 西: 270°
    /// - `distance`: 米
    /// - 返回值：点，其 x/y 是经纬度坐标
    ///
    /// # 示例
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, Destination};
    /// use geo::Point;
    ///
    /// let origin = Point::new(9.177789688110352, 48.776781529534965);
    /// let destination = Haversine::destination(origin, 45., 10000.);
    /// assert_relative_eq!(Point::new(9.274409949623532, 48.84033274015048), destination);
    /// ```
    ///
    /// # 参考文献
    ///
    /// *注意*：该实现使用了基于 [IUGG 推荐](ftp://athena.fsv.cvut.cz/ZFG/grs80-Moritz.pdf) 的 6371.088 km 的平均地球半径。
    ///
    /// [大圆]: https://en.wikipedia.org/wiki/Great_circle
    fn destination(origin: Point<F>, bearing: F, meters: F) -> Point<F> {
        let center_lng = origin.x().to_radians();
        let center_lat = origin.y().to_radians();
        let bearing_rad = bearing.to_radians();

        let rad = meters / F::from(MEAN_EARTH_RADIUS).unwrap();

        let lat =
            { center_lat.sin() * rad.cos() + center_lat.cos() * rad.sin() * bearing_rad.cos() }
                .asin();
        let lng = { bearing_rad.sin() * rad.sin() * center_lat.cos() }
            .atan2(rad.cos() - center_lat.sin() * lat.sin())
            + center_lng;

        Point::new(normalize_longitude(lng.to_degrees()), lat.to_degrees())
    }
}

impl<F: CoordFloat + FromPrimitive> Distance<F, Point<F>, Point<F>> for Haversine {
    /// 使用 [haversine 公式] 确定两点之间的距离。
    ///
    /// # 单位
    ///
    /// - `origin`, `destination`: 点，其 x/y 是经纬度坐标
    /// - 返回值：米
    ///
    /// # 示例
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, Distance};
    /// use geo::Point;
    ///
    /// let new_york_city = Point::new(-74.006f64, 40.7128f64);
    /// let london = Point::new(-0.1278f64, 51.5074f64);
    ///
    /// let distance = Haversine::distance(new_york_city, london);
    ///
    /// assert_relative_eq!(
    ///     5_570_230., // 米
    ///     distance.round()
    /// );
    /// ```
    ///
    /// # 参考文献
    ///
    /// *注意*：该实现使用了基于 [IUGG 推荐](ftp://athena.fsv.cvut.cz/ZFG/grs80-Moritz.pdf) 的 6371.088 km 的平均地球半径。
    ///
    /// [haversine 公式]: https://en.wikipedia.org/wiki/Haversine_formula
    fn distance(origin: Point<F>, destination: Point<F>) -> F {
        let two = F::one() + F::one();
        let theta1 = origin.y().to_radians();
        let theta2 = destination.y().to_radians();
        let delta_theta = (destination.y() - origin.y()).to_radians();
        let delta_lambda = (destination.x() - origin.x()).to_radians();
        let a = (delta_theta / two).sin().powi(2)
            + theta1.cos() * theta2.cos() * (delta_lambda / two).sin().powi(2);
        let c = two * a.sqrt().asin();
        F::from(MEAN_EARTH_RADIUS).unwrap() * c
    }
}

/// 沿 [大圆] 插值点。
///
/// [大圆]: https://en.wikipedia.org/wiki/Great_circle
impl<F: CoordFloat + FromPrimitive> InterpolatePoint<F> for Haversine {
    /// 返回两个现有点之间 [大圆] 上的新点。
    ///
    /// # 示例
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, InterpolatePoint};
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    ///
    /// let closer_to_p1 = Haversine::point_at_distance_between(p1, p2, 100_000.0);
    /// assert_relative_eq!(closer_to_p1, Point::new(10.81, 20.49), epsilon = 1.0e-2);
    ///
    /// let closer_to_p2 = Haversine::point_at_distance_between(p1, p2, 10_000_000.0);
    /// assert_relative_eq!(closer_to_p2, Point::new(112.33, 30.57), epsilon = 1.0e-2);
    /// ```
    ///
    /// [大圆]: https://en.wikipedia.org/wiki/Great_circle
    fn point_at_distance_between(start: Point<F>, end: Point<F>, meters_from_start: F) -> Point<F> {
        let bearing = Self::bearing(start, end);
        Self::destination(start, bearing, meters_from_start)
    }

    /// 返回两个现有点之间 [大圆] 上的新点。
    ///
    /// # 示例
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, InterpolatePoint};
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    ///
    /// let closer_to_p1 = Haversine::point_at_ratio_between(p1, p2, 0.1);
    /// assert_relative_eq!(closer_to_p1, Point::new(19.52, 25.27), epsilon = 1.0e-2);
    ///
    /// let closer_to_p2 = Haversine::point_at_ratio_between(p1, p2, 0.9);
    /// assert_relative_eq!(closer_to_p2, Point::new(114.72, 29.65), epsilon = 1.0e-2);
    ///
    /// let midpoint = Haversine::point_at_ratio_between(p1, p2, 0.5);
    /// assert_relative_eq!(midpoint, Point::new(65.87, 37.62), epsilon = 1.0e-2);
    /// ```
    ///
    /// [大圆]: https://en.wikipedia.org/wiki/Great_circle
    fn point_at_ratio_between(start: Point<F>, end: Point<F>, ratio_from_start: F) -> Point<F> {
        if start == end || ratio_from_start == F::zero() {
            return start;
        }
        if ratio_from_start == F::one() {
            return end;
        }
        let calculation = HaversineIntermediateFillCalculation::new(start, end);
        calculation.point_at_ratio(ratio_from_start)
    }

    /// 在 `start` 和 `end` 之间沿 [大圆] 插值 `Point`。
    ///
    /// 将添加尽可能多的点，以使两点之间的 [haversine 距离] 从不超过 `max_distance`。
    /// 如果起点和终点之间的距离小于 `max_distance`，则输出中不会包含其他点。
    ///
    /// `include_ends`: 是否应在输出中包含起点和终点？
    ///
    /// [大圆]: https://en.wikipedia.org/wiki/Great_circle
    /// [haversine 公式]: https://en.wikipedia.org/wiki/Haversine_formula
    fn points_along_line(
        start: Point<F>,
        end: Point<F>,
        max_distance: F,
        include_ends: bool,
    ) -> impl Iterator<Item = Point<F>> {
        let calculation = HaversineIntermediateFillCalculation::new(start, end);
        let HaversineIntermediateFillCalculation { d, .. } = calculation;

        let total_distance = d * F::from(MEAN_EARTH_RADIUS).unwrap();

        if total_distance <= max_distance {
            return if include_ends {
                vec![start, end].into_iter()
            } else {
                vec![].into_iter()
            };
        }

        let number_of_points = (total_distance / max_distance).ceil();
        let interval = F::one() / number_of_points;

        let mut current_step = interval;
        let mut points = if include_ends { vec![start] } else { vec![] };

        while current_step < F::one() {
            let point = calculation.point_at_ratio(current_step);
            points.push(point);
            current_step = current_step + interval;
        }

        if include_ends {
            points.push(end);
        }

        points.into_iter()
    }
}

#[allow(clippy::many_single_char_names)]
struct HaversineIntermediateFillCalculation<T> {
    d: T,
    n: T,
    o: T,
    p: T,
    q: T,
    r: T,
    s: T,
}

impl<T: CoordFloat + FromPrimitive> HaversineIntermediateFillCalculation<T> {
    #[allow(clippy::many_single_char_names)]
    fn new(p1: Point<T>, p2: Point<T>) -> Self {
        let one = T::one();
        let two = one + one;

        let lat1 = p1.y().to_radians();
        let lon1 = p1.x().to_radians();
        let lat2 = p2.y().to_radians();
        let lon2 = p2.x().to_radians();

        let (lat1_sin, lat1_cos) = lat1.sin_cos();
        let (lat2_sin, lat2_cos) = lat2.sin_cos();
        let (lon1_sin, lon1_cos) = lon1.sin_cos();
        let (lon2_sin, lon2_cos) = lon2.sin_cos();

        let m = lat1_cos * lat2_cos;

        let n = lat1_cos * lon1_cos;
        let o = lat2_cos * lon2_cos;
        let p = lat1_cos * lon1_sin;
        let q = lat2_cos * lon2_sin;

        // haversine公式的距离的计算，包括在最终的d中
        let k =
            (((lat1 - lat2) / two).sin().powi(2) + m * ((lon1 - lon2) / two).sin().powi(2)).sqrt();

        let d = two * k.asin();

        Self {
            d,
            n,
            o,
            p,
            q,
            r: lat1_sin,
            s: lat2_sin,
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn point_at_ratio(&self, ratio_from_start: T) -> Point<T> {
        let one = T::one();

        let HaversineIntermediateFillCalculation {
            d,
            n,
            o,
            p,
            q,
            r,
            s,
        } = *self;

        // 根据比例计算沿大圆的新点
        let a = ((one - ratio_from_start) * d).sin() / d.sin();
        let b = (ratio_from_start * d).sin() / d.sin();

        let x = a * n + b * o;
        let y = a * p + b * q;
        let z = a * r + b * s;

        let lat = z.atan2(x.hypot(y));
        let lon = y.atan2(x);

        Point::new(lon.to_degrees(), lat.to_degrees())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type MetricSpace = Haversine;

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
                Point::new(-0.8993203637245415, -1.6519568738739197e-16),
                MetricSpace::destination(origin, bearing, 100_000.0)
            );
        }
    }

    mod distance {
        use super::*;

        #[test]
        fn new_york_to_london() {
            let new_york_city = Point::new(-74.006f64, 40.7128f64);
            let london = Point::new(-0.1278f64, 51.5074f64);

            let distance = MetricSpace::distance(new_york_city, london);

            assert_relative_eq!(
                5_570_230., // 米
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
            assert_relative_eq!(midpoint, Point::new(65.87394172511485, 37.61809316888599));
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
            assert_relative_eq!(route[1], Point::new(17.882467331860965, 24.435542998803793));
        }
        #[test]
        fn points_along_line_without_endpoints() {
            let start = Point::new(10.0, 20.0);
            let end = Point::new(125.0, 25.0);
            let max_dist = 1000000.0; // 米
            let route =
                MetricSpace::points_along_line(start, end, max_dist, false).collect::<Vec<_>>();
            assert_eq!(route.len(), 11);
            assert_relative_eq!(route[0], Point::new(17.882467331860965, 24.435542998803793));
        }
    }
}
