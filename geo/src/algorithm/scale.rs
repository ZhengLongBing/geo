use crate::{AffineOps, AffineTransform, BoundingRect, Coord, CoordFloat, CoordNum, Rect};

/// 一个仿射变换，用于按比例缩放几何图形。
///
/// ## 性能
///
/// 如果要执行多个变换，比如 [`Scale`],
/// [`Skew`](crate::Skew), [`Translate`](crate::Translate), 或者 [`Rotate`](crate::Rotate)，
/// 那么通过使用 [`AffineOps`] 特征将这些变换组合成一个操作来应用会更高效。
pub trait Scale<T: CoordNum> {
    /// 从几何图形的边界框中心进行缩放。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Scale;
    /// use geo::{LineString, line_string};
    ///
    /// let ls: LineString = line_string![(x: 0., y: 0.), (x: 10., y: 10.)];
    ///
    /// let scaled = ls.scale(2.);
    ///
    /// assert_eq!(scaled, line_string![
    ///     (x: -5., y: -5.),
    ///     (x: 15., y: 15.)
    /// ]);
    /// ```
    #[must_use]
    fn scale(&self, scale_factor: T) -> Self;

    /// [`scale`](Self::scale) 的可变版本
    fn scale_mut(&mut self, scale_factor: T);

    /// 从几何图形的边界框中心进行缩放，使用不同的 `x_factor` 和 `y_factor` 来扭曲几何图形的[纵横比](https://en.wikipedia.org/wiki/Aspect_ratio)。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Scale;
    /// use geo::{LineString, line_string};
    ///
    /// let ls: LineString = line_string![(x: 0., y: 0.), (x: 10., y: 10.)];
    ///
    /// let scaled = ls.scale_xy(2., 4.);
    ///
    /// assert_eq!(scaled, line_string![
    ///     (x: -5., y: -15.),
    ///     (x: 15., y: 25.)
    /// ]);
    /// ```
    #[must_use]
    fn scale_xy(&self, x_factor: T, y_factor: T) -> Self;

    /// [`scale_xy`](Self::scale_xy) 的可变版本。
    fn scale_xy_mut(&mut self, x_factor: T, y_factor: T);

    /// 围绕`origin`点缩放几何图形。
    ///
    /// 原点*通常*为几何图形的二维边界框中心，这种情况下，你可以直接使用 [`scale`](Self::scale) 或 [`scale_xy`](Self::scale_xy)，
    /// 但此方法允许你指定任意点。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo::Scale;
    /// use geo::{LineString, line_string, Coord};
    ///
    /// let ls: LineString = line_string![(x: 0., y: 0.), (x: 10., y: 10.)];
    ///
    /// let scaled = ls.scale_around_point(2., 4., Coord { x: 100., y: 100. });
    ///
    /// assert_eq!(scaled, line_string![
    ///     (x: -100., y: -300.),
    ///     (x: -80., y: -260.)
    /// ]);
    /// ```
    #[must_use]
    fn scale_around_point(&self, x_factor: T, y_factor: T, origin: impl Into<Coord<T>>) -> Self;

    /// [`scale_around_point`](Self::scale_around_point) 的可变版本。
    fn scale_around_point_mut(&mut self, x_factor: T, y_factor: T, origin: impl Into<Coord<T>>);
}

impl<T, IR, G> Scale<T> for G
where
    T: CoordFloat,
    IR: Into<Option<Rect<T>>>,
    G: Clone + AffineOps<T> + BoundingRect<T, Output = IR>,
{
    fn scale(&self, scale_factor: T) -> Self {
        self.scale_xy(scale_factor, scale_factor)
    }

    fn scale_mut(&mut self, scale_factor: T) {
        self.scale_xy_mut(scale_factor, scale_factor);
    }

    fn scale_xy(&self, x_factor: T, y_factor: T) -> Self {
        let origin = match self.bounding_rect().into() {
            Some(rect) => rect.center(),
            // 空几何图形没有边界矩形，但在这种情况下，变换是无操作的。
            None => return self.clone(),
        };
        self.scale_around_point(x_factor, y_factor, origin)
    }

    fn scale_xy_mut(&mut self, x_factor: T, y_factor: T) {
        let origin = match self.bounding_rect().into() {
            Some(rect) => rect.center(),
            // 空几何图形没有边界矩形，但在这种情况下，变换是无操作的。
            None => return,
        };
        self.scale_around_point_mut(x_factor, y_factor, origin);
    }

    fn scale_around_point(&self, x_factor: T, y_factor: T, origin: impl Into<Coord<T>>) -> Self {
        let affineop = AffineTransform::scale(x_factor, y_factor, origin);
        self.affine_transform(&affineop)
    }

    fn scale_around_point_mut(&mut self, x_factor: T, y_factor: T, origin: impl Into<Coord<T>>) {
        let affineop = AffineTransform::scale(x_factor, y_factor, origin);
        self.affine_transform_mut(&affineop)
    }
}
