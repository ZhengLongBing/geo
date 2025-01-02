use crate::{AffineOps, AffineTransform, BoundingRect, Coord, CoordFloat, CoordNum, Rect};

/// 一种通过 x 和 y 维度的角度剪切几何图形的仿射变换。
///
/// ## 性能
///
/// 如果你要执行多次变换，比如 [`Scale`](crate::Scale)、[`Skew`]、
/// [`Translate`](crate::Translate) 或者 [`Rotate`](crate::Rotate)，
/// 那么使用 [`AffineOps`] trait 合成变换并作为一个单一操作应用会更高效。
///
pub trait Skew<T: CoordNum> {
    /// 一种通过 x 和 y 维度的统一角度剪切几何图形的仿射变换。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Skew;
    /// use geo::{Polygon, polygon};
    ///
    /// let square: Polygon = polygon![
    ///     (x: 0., y: 0.),
    ///     (x: 10., y: 0.),
    ///     (x: 10., y: 10.),
    ///     (x: 0., y: 10.)
    /// ];
    ///
    /// let skewed = square.skew(30.);
    ///
    /// let expected_output: Polygon = polygon![
    ///     (x: -2.89, y: -2.89),
    ///     (x: 7.11, y: 2.89),
    ///     (x: 12.89, y: 12.89),
    ///     (x: 2.89, y: 7.11)
    /// ];
    /// approx::assert_relative_eq!(skewed, expected_output, epsilon = 1e-2);
    /// ```
    #[must_use]
    fn skew(&self, degrees: T) -> Self;

    /// [`skew`](Self::skew) 的可变版本。
    fn skew_mut(&mut self, degrees: T);

    /// 一种通过 x 和 y 维度角度剪切几何图形的仿射变换。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Skew;
    /// use geo::{Polygon, polygon};
    ///
    /// let square: Polygon = polygon![
    ///     (x: 0., y: 0.),
    ///     (x: 10., y: 0.),
    ///     (x: 10., y: 10.),
    ///     (x: 0., y: 10.)
    /// ];
    ///
    /// let skewed = square.skew_xy(30., 12.);
    ///
    /// let expected_output: Polygon = polygon![
    ///     (x: -2.89, y: -1.06),
    ///     (x: 7.11, y: 1.06),
    ///     (x: 12.89, y: 11.06),
    ///     (x: 2.89, y: 8.94)
    /// ];
    /// approx::assert_relative_eq!(skewed, expected_output, epsilon = 1e-2);
    /// ```
    #[must_use]
    fn skew_xy(&self, degrees_x: T, degrees_y: T) -> Self;

    /// [`skew_xy`](Self::skew_xy) 的可变版本。
    fn skew_xy_mut(&mut self, degrees_x: T, degrees_y: T);

    /// 一种围绕 `origin` 点通过 x 和 y 维度的角度剪切几何图形的仿射变换。
    ///
    /// 起始点通常设定为几何图形的2D边界框中心，在这种情况下可以直接使用
    /// [`skew`](Self::skew) 或者 [`skew_xy`](Self::skew_xy)，但此方法允许指定任何点。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Skew;
    /// use geo::{Polygon, polygon, point};
    ///
    /// let square: Polygon = polygon![
    ///     (x: 0., y: 0.),
    ///     (x: 10., y: 0.),
    ///     (x: 10., y: 10.),
    ///     (x: 0., y: 10.)
    /// ];
    ///
    /// let origin = point! { x: 2., y: 2. };
    /// let skewed = square.skew_around_point(45.0, 10.0, origin);
    ///
    /// let expected_output: Polygon = polygon![
    ///     (x: -2., y: -0.353),
    ///     (x: 8., y: 1.410),
    ///     (x: 18., y: 11.41),
    ///     (x: 8., y: 9.647)
    /// ];
    /// approx::assert_relative_eq!(skewed, expected_output, epsilon = 1e-2);
    /// ```
    #[must_use]
    fn skew_around_point(&self, degrees_x: T, degrees_y: T, origin: impl Into<Coord<T>>) -> Self;

    /// [`skew_around_point`](Self::skew_around_point) 的可变版本。
    fn skew_around_point_mut(&mut self, degrees_x: T, degrees_y: T, origin: impl Into<Coord<T>>);
}

impl<T, IR, G> Skew<T> for G
where
    T: CoordFloat,
    IR: Into<Option<Rect<T>>>,
    G: Clone + AffineOps<T> + BoundingRect<T, Output = IR>,
{
    fn skew(&self, degrees: T) -> Self {
        self.skew_xy(degrees, degrees)
    }

    fn skew_mut(&mut self, degrees: T) {
        self.skew_xy_mut(degrees, degrees);
    }

    fn skew_xy(&self, degrees_x: T, degrees_y: T) -> Self {
        let origin = match self.bounding_rect().into() {
            Some(rect) => rect.center(),
            // 空几何图形没有边界框，但在这种情况下，变换无效。
            None => return self.clone(),
        };
        self.skew_around_point(degrees_x, degrees_y, origin)
    }

    fn skew_xy_mut(&mut self, degrees_x: T, degrees_y: T) {
        let origin = match self.bounding_rect().into() {
            Some(rect) => rect.center(),
            // 空几何图形没有边界框，但在这种情况下，变换无效。
            None => return,
        };
        self.skew_around_point_mut(degrees_x, degrees_y, origin);
    }

    fn skew_around_point(&self, xs: T, ys: T, origin: impl Into<Coord<T>>) -> Self {
        let transform = AffineTransform::skew(xs, ys, origin);
        self.affine_transform(&transform)
    }

    fn skew_around_point_mut(&mut self, xs: T, ys: T, origin: impl Into<Coord<T>>) {
        let transform = AffineTransform::skew(xs, ys, origin);
        self.affine_transform_mut(&transform);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{line_string, BoundingRect, Centroid, LineString};

    #[test]
    fn skew_linestring() {
        let ls: LineString<f64> = line_string![
            (x: 3.0, y: 0.0),
            (x: 3.0, y: 10.0),
        ];
        let origin = ls.bounding_rect().unwrap().centroid();
        let sheared = ls.skew_around_point(45.0, 45.0, origin);
        assert_eq!(
            sheared,
            line_string![
                (x: -1.9999999999999991, y: 0.0),
                (x: 7.999999999999999, y: 10.0)
            ]
        );
    }
}
