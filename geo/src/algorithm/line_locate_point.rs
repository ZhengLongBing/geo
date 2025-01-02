// 此算法将在未来被弃用，替换为统一的实现，而不是特定于欧几里得的。
// 在替代方案可用之前，我们允许弃用，以便不改变现有用户的方法签名。
#[allow(deprecated)]
use crate::{
    CoordFloat, Line, LineString, Point,
    {euclidean_distance::EuclideanDistance, euclidean_length::EuclideanLength},
};
use std::ops::AddAssign;

/// 返回表示线上最接近给定点的点的位置的线总长度的（可选的）分数。
///
/// 如果线段长度为零，则返回的分数字为零。
///
/// 如果点的坐标或线段的任何坐标不是有限的，则返回 `None`。
///
/// # 示例
///
/// ```
/// use geo::{LineString, point};
/// use geo::LineLocatePoint;
///
/// let linestring: LineString = vec![
///     [-1.0, 0.0],
///     [0.0, 0.0],
///     [0.0, 1.0]
/// ].into();
///
/// assert_eq!(linestring.line_locate_point(&point!(x: -1.0, y: 0.0)), Some(0.0));
/// assert_eq!(linestring.line_locate_point(&point!(x: -0.5, y: 0.0)), Some(0.25));
/// assert_eq!(linestring.line_locate_point(&point!(x: 0.0, y: 0.0)), Some(0.5));
/// assert_eq!(linestring.line_locate_point(&point!(x: 0.0, y: 0.5)), Some(0.75));
/// assert_eq!(linestring.line_locate_point(&point!(x: 0.0, y: 1.0)), Some(1.0));
/// ```
pub trait LineLocatePoint<T, Rhs> {
    type Output;
    type Rhs;

    fn line_locate_point(&self, p: &Rhs) -> Self::Output;
}

impl<T> LineLocatePoint<T, Point<T>> for Line<T>
where
    T: CoordFloat,
{
    type Output = Option<T>;
    type Rhs = Point<T>;

    fn line_locate_point(&self, p: &Self::Rhs) -> Self::Output {
        // 设 $s$ 为线段的起始点，$v$ 为它的方向向量。我们想找到 $l$ 使得
        // $(p - (s + lv)) \cdot v = 0$，即从 $l$ 沿线到 $p$ 的向量与 $v$
        // 垂直

        // 向量 $p - s$
        let sp: Point<_> = *p - self.start_point();

        // 线段的方向向量 $v$
        let v: Point<_> = (self.end - self.start).into();

        // $v \cdot v$
        let v_sq = v.dot(v);
        if v_sq == T::zero() {
            // 线段长度为零，返回零
            Some(T::zero())
        } else {
            // $v \cdot (p - s)$
            let v_dot_sp = v.dot(sp);
            let l = v_dot_sp / v_sq;
            if l.is_finite() {
                Some(l.max(T::zero()).min(T::one()))
            } else {
                None
            }
        }
    }
}

#[allow(deprecated)]
impl<T> LineLocatePoint<T, Point<T>> for LineString<T>
where
    T: CoordFloat + AddAssign,
    Line<T>: EuclideanDistance<T, Point<T>> + EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = Option<T>;
    type Rhs = Point<T>;

    fn line_locate_point(&self, p: &Self::Rhs) -> Self::Output {
        let total_length = (*self).euclidean_length();
        if total_length == T::zero() {
            return Some(T::zero());
        }
        let mut cum_length = T::zero();
        let mut closest_dist_to_point = T::infinity();
        let mut fraction = T::zero();
        for segment in self.lines() {
            let segment_distance_to_point = segment.euclidean_distance(p);
            let segment_length = segment.euclidean_length();
            let segment_fraction = segment.line_locate_point(p)?; // 如果任何段的分数为None，则返回None
            if segment_distance_to_point < closest_dist_to_point {
                closest_dist_to_point = segment_distance_to_point;
                fraction = (cum_length + segment_fraction * segment_length) / total_length;
            }
            cum_length += segment_length;
        }
        Some(fraction)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{coord, point};
    use num_traits::Float;

    #[test]
    fn test_line_locate_point_line() {
        // 一些有限的示例
        let line = Line::new(coord! { x: -1.0, y: 0.0 }, coord! { x: 1.0, y: 0.0 });
        let point = Point::new(0.0, 1.0);
        assert_eq!(line.line_locate_point(&point), Some(0.5));

        let point = Point::new(1.0, 1.0);
        assert_eq!(line.line_locate_point(&point), Some(1.0));

        let point = Point::new(2.0, 1.0);
        assert_eq!(line.line_locate_point(&point), Some(1.0));

        let point = Point::new(-1.0, 1.0);
        assert_eq!(line.line_locate_point(&point), Some(0.0));

        let point = Point::new(-2.0, 1.0);
        assert_eq!(line.line_locate_point(&point), Some(0.0));

        // 点包含无穷大或NaN
        let point = Point::new(Float::nan(), 1.0);
        assert_eq!(line.line_locate_point(&point), None);

        let point = Point::new(Float::infinity(), 1.0);
        assert_eq!(line.line_locate_point(&point), None);

        let point = Point::new(Float::neg_infinity(), 1.0);
        assert_eq!(line.line_locate_point(&point), None);

        // 线包含无穷大或NaN
        let line = Line::new(
            coord! { x: 0.0, y: 0.0 },
            coord! {
                x: Float::infinity(),
                y: 0.0,
            },
        );
        let point = Point::new(1000.0, 1000.0);
        assert_eq!(line.line_locate_point(&point), None);

        let line = Line::new(
            coord! { x: 0.0, y: 0.0 },
            coord! {
                x: Float::neg_infinity(),
                y: 0.0,
            },
        );
        let point = Point::new(1000.0, 1000.0);
        assert_eq!(line.line_locate_point(&point), None);

        let line = Line::new(
            coord! { x: 0.0, y: 0.0 },
            coord! {
                x: Float::nan(),
                y: 0.0,
            },
        );
        let point = Point::new(1000.0, 1000.0);
        assert_eq!(line.line_locate_point(&point), None);

        // 零长度线
        let line: Line = Line::new(coord! { x: 1.0, y: 1.0 }, coord! { x: 1.0, y: 1.0 });
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), Some(0.0));

        // 另一个具体的例子
        let line: Line = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 10.0, y: 0.0 });
        let pt = Point::new(555.0, 555.0);
        assert_eq!(line.line_locate_point(&pt), Some(1.0));
        let pt = Point::new(10.0000001, 0.0);
        assert_eq!(line.line_locate_point(&pt), Some(1.0));
        let pt = Point::new(9.0, 0.001);
        assert_eq!(line.line_locate_point(&pt), Some(0.9));
    }

    #[test]
    fn test_line_locate_point_linestring() {
        // 使用环的有限示例
        let ring: LineString = geo_test_fixtures::ring::<f64>();
        let pt = point!(x: 10.0, y: 1.0);
        assert_eq!(ring.line_locate_point(&pt), Some(0.0));

        let pt = point!(x: 10.0, y: 1.0000000000000742);
        assert_eq!(ring.line_locate_point(&pt), Some(0.9999999999999988));

        let pt = point!(x: 10.0, y: 1.0);
        assert_eq!(ring.line_locate_point(&pt), Some(0.0));

        // 点包含无穷大或NaN
        let pt = point!(x: Float::nan(), y: 1.0);
        assert_eq!(ring.line_locate_point(&pt), None);

        let pt = point!(x: Float::infinity(), y: 1.0);
        assert_eq!(ring.line_locate_point(&pt), None);

        let pt = point!(x: Float::neg_infinity(), y: 1.0);
        assert_eq!(ring.line_locate_point(&pt), None);

        // 点与两段线段等距 - 返回第一个最近点的分数
        let line: LineString = LineString::new(vec![
            (0.0, 0.0).into(),
            (1.0, 0.0).into(),
            (1.0, 1.0).into(),
            (0.0, 1.0).into(),
        ]);
        let pt = point!(x: 0.0, y: 0.5);
        assert_eq!(line.line_locate_point(&pt), Some(0.0));

        let line: LineString = LineString::new(vec![
            (1.0, 1.0).into(),
            (1.0, 1.0).into(),
            (1.0, 1.0).into(),
        ]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), Some(0.0));

        // 线包含无穷大或NaN
        let line: LineString = LineString::new(vec![
            coord! { x: 1.0, y: 1.0 },
            coord! {
                x: Float::nan(),
                y: 1.0,
            },
            coord! { x: 0.0, y: 0.0 },
        ]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), None);

        let line: LineString = LineString::new(vec![
            coord! { x: 1.0, y: 1.0 },
            coord! {
                x: Float::infinity(),
                y: 1.0,
            },
            coord! { x: 0.0, y: 0.0 },
        ]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), None);
        let line: LineString = LineString::new(vec![
            coord! { x: 1.0, y: 1.0 },
            coord! {
                x: Float::neg_infinity(),
                y: 1.0,
            },
            coord! { x: 0.0, y: 0.0 },
        ]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), None);
    }
}
