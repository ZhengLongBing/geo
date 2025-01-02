use crate::{coord, polygon, Coord, CoordFloat, CoordNum, Line, Polygon};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};

/// 一个 _轴对齐_ 的 2D 矩形，其面积由最小和最大 `Coord` 定义。
///
/// 构造函数和设置器确保最大 `Coord` 大于或等于最小值。
/// 因此，`Rect` 的宽度、高度和面积保证大于或等于零。
///
/// **注意。** 虽然 `Rect` 实现了 `MapCoords` 和 `RotatePoint` 算法特性，
/// 但预计的用法是保持轴对齐。特别是，只有整数倍 90 度的旋转，才会保持原始形状。
/// 在其他情况下，最小和最大点会被旋转或变换，并创建一个新的矩形（通过坐标交换以确保 min < max）。
///
/// # 示例
///
/// ```
/// use geo_types::{coord, Rect};
///
/// let rect = Rect::new(
///     coord! { x: 0., y: 4.},
///     coord! { x: 3., y: 10.},
/// );
///
/// assert_eq!(3., rect.width());
/// assert_eq!(6., rect.height());
/// assert_eq!(
///     coord! { x: 1.5, y: 7. },
///     rect.center()
/// );
/// ```
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rect<T: CoordNum = f64> {
    min: Coord<T>,
    max: Coord<T>,
}

impl<T: CoordNum> Rect<T> {
    /// 从两个角坐标创建一个新的矩形。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 10., y: 20. },
    ///     coord! { x: 30., y: 10. }
    /// );
    /// assert_eq!(rect.min(), coord! { x: 10., y: 10. });
    /// assert_eq!(rect.max(), coord! { x: 30., y: 20. });
    /// ```
    pub fn new<C>(c1: C, c2: C) -> Self
    where
        C: Into<Coord<T>>,
    {
        let c1 = c1.into();
        let c2 = c2.into();
        let (min_x, max_x) = if c1.x < c2.x {
            (c1.x, c2.x)
        } else {
            (c2.x, c1.x)
        };
        let (min_y, max_y) = if c1.y < c2.y {
            (c1.y, c2.y)
        } else {
            (c2.y, c1.y)
        };
        Self {
            min: coord! { x: min_x, y: min_y },
            max: coord! { x: max_x, y: max_y },
        }
    }

    #[deprecated(
        since = "0.6.2",
        note = "使用 `Rect::new`，因为 `Rect::try_new` 不会出错"
    )]
    #[allow(deprecated)]
    pub fn try_new<C>(c1: C, c2: C) -> Result<Rect<T>, InvalidRectCoordinatesError>
    where
        C: Into<Coord<T>>,
    {
        Ok(Rect::new(c1, c2))
    }

    /// 返回 `Rect` 的最小 `Coord`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 5., y: 5. },
    ///     coord! { x: 15., y: 15. },
    /// );
    ///
    /// assert_eq!(rect.min(), coord! { x: 5., y: 5. });
    /// ```
    pub fn min(self) -> Coord<T> {
        self.min
    }

    /// 设置 `Rect` 的最小坐标。
    ///
    /// # 可能的错误
    ///
    /// 如果 `min` 的 x/y 大于最大坐标的 x/y，程序会崩溃。
    pub fn set_min<C>(&mut self, min: C)
    where
        C: Into<Coord<T>>,
    {
        self.min = min.into();
        self.assert_valid_bounds();
    }

    /// 返回 `Rect` 的最大 `Coord`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 5., y: 5. },
    ///     coord! { x: 15., y: 15. },
    /// );
    ///
    /// assert_eq!(rect.max(), coord! { x: 15., y: 15. });
    /// ```
    pub fn max(self) -> Coord<T> {
        self.max
    }

    /// 设置 `Rect` 的最大坐标。
    ///
    /// # 可能的错误
    ///
    /// 如果 `max` 的 x/y 小于最小坐标的 x/y，程序会崩溃。
    pub fn set_max<C>(&mut self, max: C)
    where
        C: Into<Coord<T>>,
    {
        self.max = max.into();
        self.assert_valid_bounds();
    }

    /// 返回 `Rect` 的宽度。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 5., y: 5. },
    ///     coord! { x: 15., y: 15. },
    /// );
    ///
    /// assert_eq!(rect.width(), 10.);
    /// ```
    pub fn width(self) -> T {
        self.max().x - self.min().x
    }

    /// 返回 `Rect` 的高度。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 5., y: 5. },
    ///     coord! { x: 15., y: 15. },
    /// );
    ///
    /// assert_eq!(rect.height(), 10.);
    /// ```
    pub fn height(self) -> T {
        self.max().y - self.min().y
    }

    /// 从 `Rect` 创建一个 `Polygon`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use geo_types::{coord, Rect, polygon};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 0., y: 0. },
    ///     coord! { x: 1., y: 2. },
    /// );
    ///
    /// assert_eq!(
    ///     rect.to_polygon(),
    ///     polygon![
    ///         (x: 0., y: 0.),
    ///         (x: 0., y: 2.),
    ///         (x: 1., y: 2.),
    ///         (x: 1., y: 0.),
    ///         (x: 0., y: 0.),
    ///     ],
    /// );
    /// ```
    pub fn to_polygon(self) -> Polygon<T> {
        polygon![
            (x: self.min.x, y: self.min.y),
            (x: self.min.x, y: self.max.y),
            (x: self.max.x, y: self.max.y),
            (x: self.max.x, y: self.min.y),
            (x: self.min.x, y: self.min.y),
        ]
    }

    pub fn to_lines(&self) -> [Line<T>; 4] {
        [
            Line::new(
                coord! {
                    x: self.min.x,
                    y: self.min.y,
                },
                coord! {
                    x: self.min.x,
                    y: self.max.y,
                },
            ),
            Line::new(
                coord! {
                    x: self.min.x,
                    y: self.max.y,
                },
                coord! {
                    x: self.max.x,
                    y: self.max.y,
                },
            ),
            Line::new(
                coord! {
                    x: self.max.x,
                    y: self.max.y,
                },
                coord! {
                    x: self.max.x,
                    y: self.min.y,
                },
            ),
            Line::new(
                coord! {
                    x: self.max.x,
                    y: self.min.y,
                },
                coord! {
                    x: self.min.x,
                    y: self.min.y,
                },
            ),
        ]
    }

    /// 将矩形沿X轴拆分成两个等宽的矩形。
    ///
    /// # 示例
    ///
    /// ```
    /// let rect = geo_types::Rect::new(
    ///     geo_types::coord! { x: 0., y: 0. },
    ///     geo_types::coord! { x: 4., y: 4. },
    /// );
    ///
    /// let [rect1, rect2] = rect.split_x();
    ///
    /// assert_eq!(
    ///     geo_types::Rect::new(
    ///         geo_types::coord! { x: 0., y: 0. },
    ///         geo_types::coord! { x: 2., y: 4. },
    ///     ),
    ///     rect1,
    /// );
    /// assert_eq!(
    ///     geo_types::Rect::new(
    ///         geo_types::coord! { x: 2., y: 0. },
    ///         geo_types::coord! { x: 4., y: 4. },
    ///     ),
    ///     rect2,
    /// );
    /// ```
    pub fn split_x(self) -> [Rect<T>; 2] {
        let two = T::one() + T::one();
        let mid_x = self.min().x + self.width() / two;
        [
            Rect::new(self.min(), coord! { x: mid_x, y: self.max().y }),
            Rect::new(coord! { x: mid_x, y: self.min().y }, self.max()),
        ]
    }

    /// 将矩形沿Y轴拆分成两个等高的矩形。
    ///
    /// # 示例
    ///
    /// ```
    /// let rect = geo_types::Rect::new(
    ///     geo_types::coord! { x: 0., y: 0. },
    ///     geo_types::coord! { x: 4., y: 4. },
    /// );
    ///
    /// let [rect1, rect2] = rect.split_y();
    ///
    /// assert_eq!(
    ///     geo_types::Rect::new(
    ///         geo_types::coord! { x: 0., y: 0. },
    ///         geo_types::coord! { x: 4., y: 2. },
    ///     ),
    ///     rect1,
    /// );
    /// assert_eq!(
    ///     geo_types::Rect::new(
    ///         geo_types::coord! { x: 0., y: 2. },
    ///         geo_types::coord! { x: 4., y: 4. },
    ///     ),
    ///     rect2,
    /// );
    /// ```
    pub fn split_y(self) -> [Rect<T>; 2] {
        let two = T::one() + T::one();
        let mid_y = self.min().y + self.height() / two;
        [
            Rect::new(self.min(), coord! { x: self.max().x, y: mid_y }),
            Rect::new(coord! { x: self.min().x, y: mid_y }, self.max()),
        ]
    }

    fn assert_valid_bounds(&self) {
        if !self.has_valid_bounds() {
            panic!("{}", RECT_INVALID_BOUNDS_ERROR);
        }
    }

    fn has_valid_bounds(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y
    }
}

impl<T: CoordFloat> Rect<T> {
    /// 返回 `Rect` 的中心 `Coord`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 5., y: 5. },
    ///     coord! { x: 15., y: 15. },
    /// );
    ///
    /// assert_eq!(rect.center(), coord! { x: 10., y: 10. });
    /// ```
    pub fn center(self) -> Coord<T> {
        let two = T::one() + T::one();
        coord! {
            x: (self.max.x + self.min.x) / two,
            y: (self.max.y + self.min.y) / two,
        }
    }
}

static RECT_INVALID_BOUNDS_ERROR: &str =
    "无法创建 Rect: 'min' 坐标的 x/y 值必须小于或等于 'max' 的 x/y 值";

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for Rect<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    /// 在相对限制内进行相等性断言。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::Rect;
    ///
    /// let a = Rect::new((0.0, 0.0), (10.0, 10.0));
    /// let b = Rect::new((0.0, 0.0), (10.01, 10.0));
    ///
    /// approx::assert_relative_eq!(a, b, max_relative=0.1);
    /// approx::assert_relative_ne!(a, b, max_relative=0.0001);
    /// ```
    #[inline]
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if !self.min.relative_eq(&other.min, epsilon, max_relative) {
            return false;
        }

        if !self.max.relative_eq(&other.max, epsilon, max_relative) {
            return false;
        }

        true
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> AbsDiffEq for Rect<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    T::Epsilon: Copy,
{
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    /// 在绝对限制内进行相等性断言。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::{point, Rect};
    ///
    /// let a = Rect::new((0.0, 0.0), (10.0, 10.0));
    /// let b = Rect::new((0.0, 0.0), (10.01, 10.0));
    ///
    /// approx::abs_diff_eq!(a, b, epsilon=0.1);
    /// approx::abs_diff_ne!(a, b, epsilon=0.001);
    /// ```
    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if !self.min.abs_diff_eq(&other.min, epsilon) {
            return false;
        }

        if !self.max.abs_diff_eq(&other.max, epsilon) {
            return false;
        }

        true
    }
}

#[deprecated(
    since = "0.6.2",
    note = "使用 `Rect::new`，因为 `Rect::try_new` 不会出错"
)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InvalidRectCoordinatesError;

#[cfg(feature = "std")]
#[allow(deprecated)]
impl std::error::Error for InvalidRectCoordinatesError {}

#[allow(deprecated)]
impl core::fmt::Display for InvalidRectCoordinatesError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{RECT_INVALID_BOUNDS_ERROR}")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::coord;

    #[test]
    fn rect() {
        let rect = Rect::new((10, 10), (20, 20));
        assert_eq!(rect.min, coord! { x: 10, y: 10 });
        assert_eq!(rect.max, coord! { x: 20, y: 20 });

        let rect = Rect::new((20, 20), (10, 10));
        assert_eq!(rect.min, coord! { x: 10, y: 10 });
        assert_eq!(rect.max, coord! { x: 20, y: 20 });

        let rect = Rect::new((10, 20), (20, 10));
        assert_eq!(rect.min, coord! { x: 10, y: 10 });
        assert_eq!(rect.max, coord! { x: 20, y: 20 });
    }

    #[test]
    fn rect_width() {
        let rect = Rect::new((10, 10), (20, 20));
        assert_eq!(rect.width(), 10);
    }

    #[test]
    fn rect_height() {
        let rect = Rect::new((10., 10.), (20., 20.));
        assert_relative_eq!(rect.height(), 10.);
    }

    #[test]
    fn rect_center() {
        assert_relative_eq!(
            Rect::new((0., 10.), (10., 90.)).center(),
            Coord::from((5., 50.))
        );
        assert_relative_eq!(
            Rect::new((-42., -42.), (42., 42.)).center(),
            Coord::from((0., 0.))
        );
        assert_relative_eq!(
            Rect::new((0., 0.), (0., 0.)).center(),
            Coord::from((0., 0.))
        );
    }
}
