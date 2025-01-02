use crate::{coord, CoordNum, Point};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq, UlpsEq};

/// 用于存储二维笛卡尔平面上坐标的轻量级结构体。
///
/// 与`Point`不同（`Point`在未来可能包含额外信息，如包络、精度模型和空间参考系统信息），
/// `Coord`仅包含坐标值和访问方法。
///
/// 此类型实现了[向量空间]操作：
/// [`Add`]、[`Sub`]、[`Neg`]、[`Zero`]、
/// [`Mul<T>`][`Mul`]和[`Div<T>`][`Div`]特征。
///
/// # 语义
///
/// 此类型不表示任何地理空间基元，
/// 但用于它们的定义中。唯一的要求是
/// 它包含的坐标是有效数字
///（例如，不是`f64::NAN`）。
///
/// [向量空间]: //en.wikipedia.org/wiki/Vector_space
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coord<T: CoordNum = f64> {
    pub x: T,
    pub y: T,
}

#[deprecated(note = "重命名为 `geo_types::Coord`（或 `geo::Coord`）")]
pub type Coordinate<T = f64> = Coord<T>;

// 从元组实现 From trait
impl<T: CoordNum> From<(T, T)> for Coord<T> {
    #[inline]
    fn from(coords: (T, T)) -> Self {
        coord! {
            x: coords.0,
            y: coords.1,
        }
    }
}

// 从数组实现 From trait
impl<T: CoordNum> From<[T; 2]> for Coord<T> {
    #[inline]
    fn from(coords: [T; 2]) -> Self {
        coord! {
            x: coords[0],
            y: coords[1],
        }
    }
}

// 从 Point 实现 From trait
impl<T: CoordNum> From<Point<T>> for Coord<T> {
    #[inline]
    fn from(point: Point<T>) -> Self {
        coord! {
            x: point.x(),
            y: point.y(),
        }
    }
}

// 实现到元组的 From trait
impl<T: CoordNum> From<Coord<T>> for (T, T) {
    #[inline]
    fn from(coord: Coord<T>) -> Self {
        (coord.x, coord.y)
    }
}

// 实现到数组的 From trait
impl<T: CoordNum> From<Coord<T>> for [T; 2] {
    #[inline]
    fn from(coord: Coord<T>) -> Self {
        [coord.x, coord.y]
    }
}

impl<T: CoordNum> Coord<T> {
    /// 返回包含坐标的x/水平和y/垂直分量的元组。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::coord;
    ///
    /// let c = coord! {
    ///     x: 40.02f64,
    ///     y: 116.34,
    /// };
    /// let (x, y) = c.x_y();
    ///
    /// assert_eq!(y, 116.34);
    /// assert_eq!(x, 40.02f64);
    /// ```
    #[inline]
    pub fn x_y(&self) -> (T, T) {
        (self.x, self.y)
    }
}

use core::ops::{Add, Div, Mul, Neg, Sub};

/// 对坐标取反。
///
/// # 示例
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 1.25, y: 2.5 };
/// let q = -p;
///
/// assert_eq!(q.x, -p.x);
/// assert_eq!(q.y, -p.y);
/// ```
impl<T> Neg for Coord<T>
where
    T: CoordNum + Neg<Output = T>,
{
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        coord! {
            x: -self.x,
            y: -self.y,
        }
    }
}

/// 添加两个坐标。
///
/// # 示例
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 1.25, y: 2.5 };
/// let q = coord! { x: 1.5, y: 2.5 };
/// let sum = p + q;
///
/// assert_eq!(sum.x, 2.75);
/// assert_eq!(sum.y, 5.0);
/// ```
impl<T: CoordNum> Add for Coord<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        coord! {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

/// 从一个坐标中减去另一个坐标。
///
/// # 示例
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 1.5, y: 2.5 };
/// let q = coord! { x: 1.25, y: 2.5 };
/// let diff = p - q;
///
/// assert_eq!(diff.x, 0.25);
/// assert_eq!(diff.y, 0.);
/// ```
impl<T: CoordNum> Sub for Coord<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        coord! {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// 坐标与标量相乘。
///
/// # 示例
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 1.25, y: 2.5 };
/// let q = p * 4.;
///
/// assert_eq!(q.x, 5.0);
/// assert_eq!(q.y, 10.0);
/// ```
impl<T: CoordNum> Mul<T> for Coord<T> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: T) -> Self {
        coord! {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

/// 坐标除以标量。
///
/// # 示例
///
/// ```
/// use geo_types::coord;
///
/// let p = coord! { x: 5., y: 10. };
/// let q = p / 4.;
///
/// assert_eq!(q.x, 1.25);
/// assert_eq!(q.y, 2.5);
/// ```
impl<T: CoordNum> Div<T> for Coord<T> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: T) -> Self {
        coord! {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

use num_traits::Zero;
/// 在原点创建一个坐标。
///
/// # 示例
///
/// ```
/// use geo_types::Coord;
/// use num_traits::Zero;
///
/// let p: Coord = Zero::zero();
///
/// assert_eq!(p.x, 0.);
/// assert_eq!(p.y, 0.);
/// ```
impl<T: CoordNum> Coord<T> {
    #[inline]
    pub fn zero() -> Self {
        coord! {
            x: T::zero(),
            y: T::zero(),
        }
    }
}

// 实现 Zero trait
impl<T: CoordNum> Zero for Coord<T> {
    #[inline]
    fn zero() -> Self {
        Self::zero()
    }
    #[inline]
    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }
}

// 实现 AbsDiffEq trait
#[cfg(any(feature = "approx", test))]
impl<T: CoordNum + AbsDiffEq> AbsDiffEq for Coord<T>
where
    T::Epsilon: Copy,
{
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> T::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: T::Epsilon) -> bool {
        T::abs_diff_eq(&self.x, &other.x, epsilon) && T::abs_diff_eq(&self.y, &other.y, epsilon)
    }
}

// 实现 RelativeEq trait
#[cfg(any(feature = "approx", test))]
impl<T: CoordNum + RelativeEq> RelativeEq for Coord<T>
where
    T::Epsilon: Copy,
{
    #[inline]
    fn default_max_relative() -> T::Epsilon {
        T::default_max_relative()
    }

    #[inline]
    fn relative_eq(&self, other: &Self, epsilon: T::Epsilon, max_relative: T::Epsilon) -> bool {
        T::relative_eq(&self.x, &other.x, epsilon, max_relative)
            && T::relative_eq(&self.y, &other.y, epsilon, max_relative)
    }
}

// 实现 UlpsEq trait
#[cfg(any(feature = "approx", test))]
impl<T: CoordNum + UlpsEq> UlpsEq for Coord<T>
where
    T::Epsilon: Copy,
{
    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    #[inline]
    fn ulps_eq(&self, other: &Self, epsilon: T::Epsilon, max_ulps: u32) -> bool {
        T::ulps_eq(&self.x, &other.x, epsilon, max_ulps)
            && T::ulps_eq(&self.y, &other.y, epsilon, max_ulps)
    }
}

// 为 rstar 0.8 版本实现 Point trait
#[cfg(feature = "rstar_0_8")]
impl<T> ::rstar_0_8::Point for Coord<T>
where
    T: ::num_traits::Float + ::rstar_0_8::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    #[inline]
    fn generate(generator: impl Fn(usize) -> Self::Scalar) -> Self {
        coord! {
            x: generator(0),
            y: generator(1),
        }
    }

    #[inline]
    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}

// 为 rstar 0.9 版本实现 Point trait
#[cfg(feature = "rstar_0_9")]
impl<T> ::rstar_0_9::Point for Coord<T>
where
    T: ::num_traits::Float + ::rstar_0_9::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    #[inline]
    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        coord! {
            x: generator(0),
            y: generator(1),
        }
    }

    #[inline]
    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}

// 为 rstar 0.10 版本实现 Point trait
#[cfg(feature = "rstar_0_10")]
impl<T> ::rstar_0_10::Point for Coord<T>
where
    T: ::num_traits::Float + ::rstar_0_10::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    #[inline]
    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        coord! {
            x: generator(0),
            y: generator(1),
        }
    }

    #[inline]
    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}

// 为 rstar 0.11 版本实现 Point trait
#[cfg(feature = "rstar_0_11")]
impl<T> ::rstar_0_11::Point for Coord<T>
where
    T: ::num_traits::Float + ::rstar_0_11::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    #[inline]
    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        coord! {
            x: generator(0),
            y: generator(1),
        }
    }

    #[inline]
    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "rstar_0_12")]
impl<T> ::rstar_0_12::Point for Coord<T>
where
    T: ::num_traits::Float + ::rstar_0_12::RTreeNum,
{
    type Scalar = T;

    const DIMENSIONS: usize = 2;

    #[inline]
    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        coord! {
            x: generator(0),
            y: generator(1),
        }
    }

    #[inline]
    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}
