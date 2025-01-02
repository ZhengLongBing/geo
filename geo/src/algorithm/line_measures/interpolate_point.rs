use crate::{CoordFloat, Point};

/// 在两个已存在点之间的线上插值一个`Point`
pub trait InterpolatePoint<F: CoordFloat> {
    /// 返回在两个已存在点之间的线上插值出的一个新点。
    ///
    /// 详见[具体实现](#implementors)。
    fn point_at_distance_between(
        start: Point<F>,
        end: Point<F>,
        distance_from_start: F,
    ) -> Point<F>;

    /// 返回在两个已存在点之间的线上插值出的一个新点。
    ///
    /// 详见[具体实现](#implementors)。
    fn point_at_ratio_between(start: Point<F>, end: Point<F>, ratio_from_start: F) -> Point<F>;

    /// 插值出在`start`和`end`之间的`Point`。
    ///
    /// 详见[具体实现](#implementors)。
    ///
    /// 将根据需要添加点，以确保点之间的距离不超过`max_distance`。
    /// 如果起始和终点之间的距离小于`max_distance`，则输出中不包含附加点。
    ///
    /// `include_ends`: 是否将起点和终点包含在输出中？
    fn points_along_line(
        start: Point<F>,
        end: Point<F>,
        max_distance: F,
        include_ends: bool,
    ) -> impl Iterator<Item = Point<F>>;
}

#[cfg(test)]
mod tests {
    use crate::{Euclidean, Geodesic, Haversine, InterpolatePoint, Point, Rhumb};

    #[test]
    fn point_at_ratio_between_line_ends() {
        let start = Point::new(0.0, 0.0);
        let end = Point::new(1.0, 1.0);

        let ratio = 0.0;
        assert_eq!(Haversine::point_at_ratio_between(start, end, ratio), start);
        assert_eq!(Euclidean::point_at_ratio_between(start, end, ratio), start);
        assert_eq!(Geodesic::point_at_ratio_between(start, end, ratio), start);
        assert_eq!(Rhumb::point_at_ratio_between(start, end, ratio), start);

        let ratio = 1.0;
        assert_eq!(Haversine::point_at_ratio_between(start, end, ratio), end);
        assert_eq!(Euclidean::point_at_ratio_between(start, end, ratio), end);
        assert_eq!(Geodesic::point_at_ratio_between(start, end, ratio), end);
        assert_eq!(Rhumb::point_at_ratio_between(start, end, ratio), end);
    }

    mod degenerate {
        use super::*;

        #[test]
        fn point_at_ratio_between_collapsed_line() {
            let start = Point::new(1.0, 1.0);

            let ratio = 0.0;
            assert_eq!(
                Haversine::point_at_ratio_between(start, start, ratio),
                start
            );
            assert_eq!(
                Euclidean::point_at_ratio_between(start, start, ratio),
                start
            );
            assert_eq!(Geodesic::point_at_ratio_between(start, start, ratio), start);
            assert_eq!(Rhumb::point_at_ratio_between(start, start, ratio), start);

            let ratio = 0.5;
            assert_eq!(
                Haversine::point_at_ratio_between(start, start, ratio),
                start
            );
            assert_eq!(
                Euclidean::point_at_ratio_between(start, start, ratio),
                start
            );
            assert_eq!(Geodesic::point_at_ratio_between(start, start, ratio), start);
            assert_eq!(Rhumb::point_at_ratio_between(start, start, ratio), start);

            let ratio = 1.0;
            assert_eq!(
                Haversine::point_at_ratio_between(start, start, ratio),
                start
            );
            assert_eq!(
                Euclidean::point_at_ratio_between(start, start, ratio),
                start
            );
            assert_eq!(Geodesic::point_at_ratio_between(start, start, ratio), start);
            assert_eq!(Rhumb::point_at_ratio_between(start, start, ratio), start);
        }

        #[test]
        fn point_at_distance_between_collapsed_line() {
            // 此方法仅记录现有行为。我认为我们当前的行为不是特别有用，但我们可能会考虑某天统一处理。
            let start: Point = Point::new(1.0, 1.0);

            let distance = 0.0;
            assert_eq!(
                Haversine::point_at_distance_between(start, start, distance),
                start
            );

            let euclidean_result = Euclidean::point_at_distance_between(start, start, distance);
            assert!(euclidean_result.x().is_nan());
            assert!(euclidean_result.y().is_nan());
            assert_eq!(
                Geodesic::point_at_distance_between(start, start, distance),
                start
            );
            assert_eq!(
                Rhumb::point_at_distance_between(start, start, distance),
                start
            );

            let distance = 100000.0;
            let due_north = Point::new(1.0, 1.9);
            let due_south = Point::new(1.0, 0.1);
            assert_relative_eq!(
                Haversine::point_at_distance_between(start, start, distance),
                due_north,
                epsilon = 1.0e-1
            );
            let euclidean_result = Euclidean::point_at_distance_between(start, start, distance);
            assert!(euclidean_result.x().is_nan());
            assert!(euclidean_result.y().is_nan());
            assert_relative_eq!(
                Geodesic::point_at_distance_between(start, start, distance),
                due_south,
                epsilon = 1.0e-1
            );
            assert_relative_eq!(
                Rhumb::point_at_distance_between(start, start, distance),
                due_north,
                epsilon = 1.0e-1
            );
        }

        #[test]
        fn points_along_collapsed_line() {
            let start = Point::new(1.0, 1.0);

            let max_distance = 1.0;

            let include_ends = true;
            let points: Vec<_> =
                Haversine::points_along_line(start, start, max_distance, include_ends).collect();
            assert_eq!(points, vec![start, start]);

            let points: Vec<_> =
                Euclidean::points_along_line(start, start, max_distance, include_ends).collect();
            assert_eq!(points, vec![start, start]);

            let points: Vec<_> =
                Geodesic::points_along_line(start, start, max_distance, include_ends).collect();
            assert_eq!(points, vec![start, start]);

            let points: Vec<_> =
                Rhumb::points_along_line(start, start, max_distance, include_ends).collect();
            assert_eq!(points, vec![start, start]);

            let include_ends = false;
            let points: Vec<_> =
                Haversine::points_along_line(start, start, max_distance, include_ends).collect();
            assert_eq!(points, vec![]);

            let points: Vec<_> =
                Euclidean::points_along_line(start, start, max_distance, include_ends).collect();
            assert_eq!(points, vec![]);

            let points: Vec<_> =
                Geodesic::points_along_line(start, start, max_distance, include_ends).collect();
            assert_eq!(points, vec![]);

            let points: Vec<_> =
                Rhumb::points_along_line(start, start, max_distance, include_ends).collect();
            assert_eq!(points, vec![]);
        }
    }
}
