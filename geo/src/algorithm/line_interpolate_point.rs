use crate::coords_iter::CoordsIter;
// 此算法未来将被弃用，将由一个统一的实现来代替，而不是仅限于欧几里得特定实现。
// 在可替代方案可用之前，我们允许使用弃用的代码，以避免更改现有用户的方法签名。
#[allow(deprecated)]
use crate::{CoordFloat, EuclideanLength, Line, LineString, Point};
use std::ops::AddAssign;

/// 返回线段上某个给定分数点的选项。
///
/// 如果给定的分数
///  * 小于零（包括负无穷）：返回起始点的 `Some`
///  * 大于一（包括无穷）：返回终点的 `Some`
///
/// 如果分数是 NaN 或者线段的任何坐标不是有限的，返回 `None`。
///
/// # 示例
///
/// ```
/// use geo::{LineString, point};
/// use geo::LineInterpolatePoint;
///
/// let linestring: LineString = vec![
///     [-1.0, 0.0],
///     [0.0, 0.0],
///     [0.0, 1.0]
/// ].into();
///
/// assert_eq!(linestring.line_interpolate_point(-1.0), Some(point!(x: -1.0, y: 0.0)));
/// assert_eq!(linestring.line_interpolate_point(0.25), Some(point!(x: -0.5, y: 0.0)));
/// assert_eq!(linestring.line_interpolate_point(0.5), Some(point!(x: 0.0, y: 0.0)));
/// assert_eq!(linestring.line_interpolate_point(0.75), Some(point!(x: 0.0, y: 0.5)));
/// assert_eq!(linestring.line_interpolate_point(2.0), Some(point!(x: 0.0, y: 1.0)));
/// ```
pub trait LineInterpolatePoint<F: CoordFloat> {
    type Output;

    fn line_interpolate_point(&self, fraction: F) -> Self::Output;
}

impl<T> LineInterpolatePoint<T> for Line<T>
where
    T: CoordFloat,
{
    type Output = Option<Point<T>>;

    fn line_interpolate_point(&self, fraction: T) -> Self::Output {
        if (fraction >= T::zero()) && (fraction <= T::one()) {
            // 分数在0到1之间，返回起点和终点之间的点
            let diff = self.end - self.start;
            let r = self.start + diff * (fraction);
            if r.x.is_finite() && r.y.is_finite() {
                Some(r.into())
            } else {
                None
            }
        } else if fraction < T::zero() {
            // 负分数替换为零
            self.line_interpolate_point(T::zero())
        } else if fraction > T::one() {
            // 大于1的分数替换为1
            self.line_interpolate_point(T::one())
        } else {
            // 分数是NaN
            debug_assert!(fraction.is_nan());
            None
        }
    }
}

#[allow(deprecated)]
impl<T> LineInterpolatePoint<T> for LineString<T>
where
    T: CoordFloat + AddAssign + std::fmt::Debug,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = Option<Point<T>>;

    fn line_interpolate_point(&self, fraction: T) -> Self::Output {
        if (fraction >= T::zero()) && (fraction <= T::one()) {
            // 找到线串中某个分数点所在的位置
            let total_length = self.euclidean_length();
            let fractional_length = total_length * fraction;
            let mut cum_length = T::zero();
            for segment in self.lines() {
                let length = segment.euclidean_length();
                if cum_length + length >= fractional_length {
                    let segment_fraction = (fractional_length - cum_length) / length;
                    return segment.line_interpolate_point(segment_fraction);
                }
                cum_length += length;
            }
            // 如果 cum_length + length 永远不大于 fractional_length，意味着 fractional_length 是 nan，或者线串没有线段可循环
            debug_assert!(fractional_length.is_nan() || (self.coords_count() == 0));
            None
        } else if fraction < T::zero() {
            // 负分数替换为零
            self.line_interpolate_point(T::zero())
        } else if fraction > T::one() {
            // 大于1的分数替换为1
            self.line_interpolate_point(T::one())
        } else {
            // 分数是NaN
            debug_assert!(fraction.is_nan());
            None
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{coord, point};
    use crate::{ClosestPoint, LineLocatePoint};
    use num_traits::Float;

    #[test]
    fn test_line_interpolate_point_line() {
        let line = Line::new(coord! { x: -1.0, y: 0.0 }, coord! { x: 1.0, y: 0.0 });
        // 一些有限的示例
        assert_eq!(
            line.line_interpolate_point(-1.0),
            Some(point!(x: -1.0, y: 0.0))
        );
        assert_eq!(
            line.line_interpolate_point(0.5),
            Some(point!(x: 0.0, y: 0.0))
        );
        assert_eq!(
            line.line_interpolate_point(0.75),
            Some(point!(x: 0.5, y: 0.0))
        );
        assert_eq!(
            line.line_interpolate_point(0.0),
            Some(point!(x: -1.0, y: 0.0))
        );
        assert_eq!(
            line.line_interpolate_point(1.0),
            Some(point!(x: 1.0, y: 0.0))
        );
        assert_eq!(
            line.line_interpolate_point(2.0),
            Some(point!(x: 1.0, y: 0.0))
        );

        // 分数是nan或inf
        assert_eq!(line.line_interpolate_point(Float::nan()), None);
        assert_eq!(
            line.line_interpolate_point(Float::infinity()),
            Some(line.end_point())
        );
        assert_eq!(
            line.line_interpolate_point(Float::neg_infinity()),
            Some(line.start_point())
        );

        let line = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 1.0, y: 1.0 });
        assert_eq!(
            line.line_interpolate_point(0.5),
            Some(point!(x: 0.5, y: 0.5))
        );

        // 线段包含nans或infs
        let line = Line::new(
            coord! {
                x: Float::nan(),
                y: 0.0,
            },
            coord! { x: 1.0, y: 1.0 },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);

        let line = Line::new(
            coord! {
                x: Float::infinity(),
                y: 0.0,
            },
            coord! { x: 1.0, y: 1.0 },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);

        let line = Line::new(
            coord! { x: 0.0, y: 0.0 },
            coord! {
                x: 1.0,
                y: Float::infinity(),
            },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);

        let line = Line::new(
            coord! {
                x: Float::neg_infinity(),
                y: 0.0,
            },
            coord! { x: 1.0, y: 1.0 },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);

        let line = Line::new(
            coord! { x: 0.0, y: 0.0 },
            coord! {
                x: 1.0,
                y: Float::neg_infinity(),
            },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);
    }

    #[test]
    fn test_line_interpolate_point_linestring() {
        // 一些有限的示例
        let linestring: LineString = vec![[-1.0, 0.0], [0.0, 0.0], [1.0, 0.0]].into();
        assert_eq!(
            linestring.line_interpolate_point(0.0),
            Some(point!(x: -1.0, y: 0.0))
        );
        assert_eq!(
            linestring.line_interpolate_point(0.5),
            Some(point!(x: 0.0, y: 0.0))
        );
        assert_eq!(
            linestring.line_interpolate_point(1.0),
            Some(point!(x: 1.0, y: 0.0))
        );
        assert_eq!(
            linestring.line_interpolate_point(1.0),
            linestring.line_interpolate_point(2.0)
        );
        assert_eq!(
            linestring.line_interpolate_point(0.0),
            linestring.line_interpolate_point(-2.0)
        );

        // 分数是nan或inf
        assert_eq!(
            linestring.line_interpolate_point(Float::infinity()),
            linestring.points().last()
        );
        assert_eq!(
            linestring.line_interpolate_point(Float::neg_infinity()),
            linestring.points().next()
        );
        assert_eq!(linestring.line_interpolate_point(Float::nan()), None);

        let linestring: LineString = vec![[-1.0, 0.0], [0.0, 0.0], [0.0, 1.0]].into();
        assert_eq!(
            linestring.line_interpolate_point(0.5),
            Some(point!(x: 0.0, y: 0.0))
        );
        assert_eq!(
            linestring.line_interpolate_point(1.5),
            Some(point!(x: 0.0, y: 1.0))
        );

        // 包含 nans/infs 的线串
        let linestring: LineString = vec![[-1.0, 0.0], [0.0, Float::nan()], [0.0, 1.0]].into();
        assert_eq!(linestring.line_interpolate_point(0.5), None);
        assert_eq!(linestring.line_interpolate_point(1.5), None);
        assert_eq!(linestring.line_interpolate_point(-1.0), None);

        let linestring: LineString = vec![[-1.0, 0.0], [0.0, Float::infinity()], [0.0, 1.0]].into();
        assert_eq!(linestring.line_interpolate_point(0.5), None);
        assert_eq!(linestring.line_interpolate_point(1.5), None);
        assert_eq!(linestring.line_interpolate_point(-1.0), None);

        let linestring: LineString =
            vec![[-1.0, 0.0], [0.0, Float::neg_infinity()], [0.0, 1.0]].into();
        assert_eq!(linestring.line_interpolate_point(0.5), None);
        assert_eq!(linestring.line_interpolate_point(1.5), None);
        assert_eq!(linestring.line_interpolate_point(-1.0), None);

        // 空线
        let coords: Vec<Point> = Vec::new();
        let linestring: LineString = coords.into();
        assert_eq!(linestring.line_interpolate_point(0.5), None);
    }

    #[test]
    fn test_matches_closest_point() {
        // line_locate_point 应该返回最接近点的分数，
        // 因此使用该分数对线进行插值应产生最接近点
        let linestring: LineString = vec![[-1.0, 0.0], [0.5, 1.0], [1.0, 2.0]].into();
        let pt = point!(x: 0.7, y: 0.7);
        let frac = linestring
            .line_locate_point(&pt)
            .expect("应返回0和1之间的分数");
        let interpolated_point = linestring
            .line_interpolate_point(frac)
            .expect("不应返回None");
        let closest_point = linestring.closest_point(&pt);
        match closest_point {
            crate::Closest::SinglePoint(p) => assert_eq!(interpolated_point, p),
            _ => panic!("最近的点应该是一个SinglePoint"), // 示例选择不作为交叉点
        };
    }
}
