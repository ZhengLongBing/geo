use num_traits::ToPrimitive;

use crate::{Coord, CoordFloat, CoordNum, MapCoords, MapCoordsInPlace};
use std::{fmt, ops::Mul, ops::Neg};

/// 应用[`AffineTransform`]如[`scale`](AffineTransform::scale)、[`skew`](AffineTransform::skew)或[`rotate`](AffineTransform::rotate)到[`Geometry`](crate::geometry::Geometry)。
///
/// 多种变换可以组合成一个单一操作进行高效应用。请参阅[`AffineTransform`]以了解更多关于如何建立转换的信息。
///
/// 如果您不进行操作组合，存在利用同一机制的特征（trait），这些特征可能更具可读性。参见：[`Scale`](crate::algorithm::Scale)，[`Translate`](crate::algorithm::Translate)，[`Rotate`](crate::algorithm::Rotate)，和[`Skew`](crate::algorithm::Skew)。
///
/// # 示例
/// ## 从构造函数开始构建变换，然后链式调用变异操作
/// ```
/// use geo::{AffineOps, AffineTransform};
/// use geo::{point, line_string, BoundingRect};
/// use approx::assert_relative_eq;
///
/// let line_string = line_string![(x: 0.0, y: 0.0),(x: 1.0, y: 1.0)];
///
/// let transform = AffineTransform::translate(1.0, 1.0).scaled(2.0, 2.0, point!(x: 0.0, y: 0.0));
///
/// let transformed_line_string = line_string.affine_transform(&transform);
///
/// assert_relative_eq!(
///     transformed_line_string,
///     line_string![(x: 2.0, y: 2.0),(x: 4.0, y: 4.0)]
/// );
/// ```
pub trait AffineOps<T: CoordNum> {
    /// 不可变地应用 `transform`，输出一个新的几何体。
    #[must_use]
    fn affine_transform(&self, transform: &AffineTransform<T>) -> Self;

    /// 应用 `transform` 以改变 `self`。
    fn affine_transform_mut(&mut self, transform: &AffineTransform<T>);
}

impl<T: CoordNum, M: MapCoordsInPlace<T> + MapCoords<T, T, Output = Self>> AffineOps<T> for M {
    fn affine_transform(&self, transform: &AffineTransform<T>) -> Self {
        self.map_coords(|c| transform.apply(c))
    }

    fn affine_transform_mut(&mut self, transform: &AffineTransform<T>) {
        self.map_coords_in_place(|c| transform.apply(c))
    }
}

/// 一个通用的仿射变换矩阵及相关操作。
///
/// 请注意，仿射操作已经在大多数 `geo-types` 基元上实现，使用此模块。
///
/// 使用相同数字类型（例如：[`CoordFloat`]）的仿射变换可以被组合，其结果可以应用于几何图形，例如使用 [`MapCoords`]。这允许变换的高效应用：可以链式调用任意数量的操作。然后将它们组合，生成一个最终的变换矩阵并应用于几何坐标。
///
/// `AffineTransform` 是一个行优先存储的矩阵。
/// 2D 仿射变换需要六个矩阵参数：
///
/// `[a, b, xoff, d, e, yoff]`
///
/// 这些参数映射到 `AffineTransform` 的行如下：
/// ```ignore
/// [[a, b, xoff],
/// [d, e, yoff],
/// [0, 0, 1]]
/// ```
/// 转换坐标 (x, y) -> (x', y') 的方程如下：
///
/// `x' = ax + by + xoff`
///
/// `y' = dx + ey + yoff`
///
/// # 用法
///
/// 提供了两种操作类型：构造和变异。**构造**函数创建一个*新的*变换，并用**现在时**表示：`scale()`、`translate()`、`rotate()` 和 `skew()`。
///
/// **变异**方法*添加*变换到现有的 `AffineTransform` 中，并且用过去分词形式表示：`scaled()`、`translated()`、`rotated()` 和 `skewed()`。
///
/// # 示例
/// ## 从构造函数开始构建变换，然后链式调用变异操作
/// ```
/// use geo::{AffineOps, AffineTransform};
/// use geo::{point, line_string, BoundingRect};
/// use approx::assert_relative_eq;
///
/// let line_string = line_string![(x: 0.0, y: 0.0),(x: 1.0, y: 1.0)];
///
/// let transform = AffineTransform::translate(1.0, 1.0).scaled(2.0, 2.0, point!(x: 0.0, y: 0.0));
///
/// let transformed_line_string = line_string.affine_transform(&transform);
///
/// assert_relative_eq!(
///     transformed_line_string,
///     line_string![(x: 2.0, y: 2.0),(x: 4.0, y: 4.0)]
/// );
/// ```
///
/// ## 手动创建仿射变换，并使用获取器方法访问元素
/// ```
/// use geo::AffineTransform;
///
/// let transform = AffineTransform::new(10.0, 0.0, 400_000.0, 0.0, -10.0, 500_000.0);
///
/// let a: f64 = transform.a();
/// let b: f64 = transform.b();
/// let xoff: f64 = transform.xoff();
/// let d: f64 = transform.d();
/// let e: f64 = transform.e();
/// let yoff: f64 = transform.yoff();
/// assert_eq!(transform, AffineTransform::new(a, b, xoff, d, e, yoff))
/// ```

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct AffineTransform<T: CoordNum = f64>([[T; 3]; 3]);

impl<T: CoordNum> Default for AffineTransform<T> {
    fn default() -> Self {
        // 单位矩阵
        Self::identity()
    }
}

impl<T: CoordNum> AffineTransform<T> {
    /// 通过组合两个 `AffineTransform` 创建一个新的仿射变换。
    ///
    /// 这是一个**累积**操作；新变换*添加*到现有变换中。
    #[must_use]
    pub fn compose(&self, other: &Self) -> Self {
        // 哈哈
        Self([
            [
                (other.0[0][0] * self.0[0][0])
                    + (other.0[0][1] * self.0[1][0])
                    + (other.0[0][2] * self.0[2][0]),
                (other.0[0][0] * self.0[0][1])
                    + (other.0[0][1] * self.0[1][1])
                    + (other.0[0][2] * self.0[2][1]),
                (other.0[0][0] * self.0[0][2])
                    + (other.0[0][1] * self.0[1][2])
                    + (other.0[0][2] * self.0[2][2]),
            ],
            [
                (other.0[1][0] * self.0[0][0])
                    + (other.0[1][1] * self.0[1][0])
                    + (other.0[1][2] * self.0[2][0]),
                (other.0[1][0] * self.0[0][1])
                    + (other.0[1][1] * self.0[1][1])
                    + (other.0[1][2] * self.0[2][1]),
                (other.0[1][0] * self.0[0][2])
                    + (other.0[1][1] * self.0[1][2])
                    + (other.0[1][2] * self.0[2][2]),
            ],
            [
                // 这一部分技术上不是必须的，因为最后一行是不变的：[0, 0, 1]
                (other.0[2][0] * self.0[0][0])
                    + (other.0[2][1] * self.0[1][0])
                    + (other.0[2][2] * self.0[2][0]),
                (other.0[2][0] * self.0[0][1])
                    + (other.0[2][1] * self.0[1][1])
                    + (other.0[2][2] * self.0[2][1]),
                (other.0[2][0] * self.0[0][2])
                    + (other.0[2][1] * self.0[1][2])
                    + (other.0[2][2] * self.0[2][2]),
            ],
        ])
    }

    /// 通过组合任意数量的 `AffineTransform` 创建一个新的仿射变换。
    ///
    /// 这是一个**累积**操作；新变换*添加*到现有变换中。
    /// ```
    /// use geo::AffineTransform;
    /// let mut transform = AffineTransform::identity();
    ///
    /// // 创建两个相互抵消的变换
    /// let transform1 = AffineTransform::translate(1.0, 2.0);
    /// let transform2 = AffineTransform::translate(-1.0, -2.0);
    /// let transforms = vec![transform1, transform2];
    ///
    /// // 应用它们
    /// let outcome = transform.compose_many(&transforms);
    /// // 我们应该回到起点
    /// assert!(outcome.is_identity());
    /// ```
    #[must_use]
    pub fn compose_many(&self, transforms: &[Self]) -> Self {
        self.compose(&transforms.iter().fold(
            AffineTransform::default(),
            |acc: AffineTransform<T>, transform| acc.compose(transform),
        ))
    }

    /// 创建单位矩阵
    ///
    /// 矩阵为：
    /// ```ignore
    /// [[1, 0, 0],
    /// [0, 1, 0],
    /// [0, 0, 1]]
    /// ```
    pub fn identity() -> Self {
        Self::new(
            T::one(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::one(),
            T::zero(),
        )
    }

    /// 判断变换是否等价于[单位矩阵](Self::identity)，也就是说，应用它是否是无效操作。
    ///
    /// ```
    /// use geo::AffineTransform;
    /// let mut transform = AffineTransform::identity();
    /// assert!(transform.is_identity());
    ///
    /// // 稍微变换一下
    /// transform = transform.translated(1.0, 2.0);
    /// assert!(!transform.is_identity());
    ///
    /// // 将其复原
    /// transform = transform.translated(-1.0, -2.0);
    /// assert!(transform.is_identity());
    /// ```
    pub fn is_identity(&self) -> bool {
        self == &Self::identity()
    }

    /// **创建**一个缩放的仿射变换，在 `x` 和 `y` 维度上按比例缩放。
    /// 原点通常是几何图形的2D边界框中心，但可以指定任何坐标。
    /// 负缩放因子将镜像或反映坐标。
    ///
    /// 矩阵为：
    /// ```ignore
    /// [[xfact, 0, xoff],
    /// [0, yfact, yoff],
    /// [0, 0, 1]]
    ///
    /// xoff = origin.x - (origin.x * xfact)
    /// yoff = origin.y - (origin.y * yfact)
    /// ```
    pub fn scale(xfact: T, yfact: T, origin: impl Into<Coord<T>>) -> Self {
        let (x0, y0) = origin.into().x_y();
        let xoff = x0 - (x0 * xfact);
        let yoff = y0 - (y0 * yfact);
        Self::new(xfact, T::zero(), xoff, T::zero(), yfact, yoff)
    }

    /// **添加**缩放的仿射变换，在 `x` 和 `y` 维度上按比例缩放。
    /// 原点通常是几何图形的2D边界框中心，但可以指定任何坐标。
    /// 负缩放因子将会镜像或反射坐标。
    /// 这是一个**累积**操作；新变换*添加*到现有变换中。
    #[must_use]
    pub fn scaled(mut self, xfact: T, yfact: T, origin: impl Into<Coord<T>>) -> Self {
        self.0 = self.compose(&Self::scale(xfact, yfact, origin)).0;
        self
    }

    /// **创建**平移的仿射变换，在 `x` 和 `y` 维度上通过偏移进行移动。
    ///
    /// 矩阵为：
    /// ```ignore
    /// [[1, 0, xoff],
    /// [0, 1, yoff],
    /// [0, 0, 1]]
    /// ```
    pub fn translate(xoff: T, yoff: T) -> Self {
        Self::new(T::one(), T::zero(), xoff, T::zero(), T::one(), yoff)
    }

    /// **添加**平移的仿射变换，在 `x` 和 `y` 维度上通过偏移进行移动。
    ///
    /// 这是一个**累积**操作；新变换*添加*到现有变换中。
    #[must_use]
    pub fn translated(mut self, xoff: T, yoff: T) -> Self {
        self.0 = self.compose(&Self::translate(xoff, yoff)).0;
        self
    }

    /// 应用当前变换到一个坐标
    pub fn apply(&self, coord: Coord<T>) -> Coord<T> {
        Coord {
            x: (self.0[0][0] * coord.x + self.0[0][1] * coord.y + self.0[0][2]),
            y: (self.0[1][0] * coord.x + self.0[1][1] * coord.y + self.0[1][2]),
        }
    }

    /// 创建一个新的自定义变换矩阵
    ///
    /// 参数顺序与仿射变换矩阵一致：
    ///```ignore
    /// [[a, b, xoff],
    ///  [d, e, yoff],
    ///  [0, 0, 1]] <-- 不属于输入参数
    /// ```
    pub fn new(a: T, b: T, xoff: T, d: T, e: T, yoff: T) -> Self {
        Self([[a, b, xoff], [d, e, yoff], [T::zero(), T::zero(), T::one()]])
    }

    /// 请参阅 [AffineTransform::new] 以了解该值在仿射变换中的作用。
    pub fn a(&self) -> T {
        self.0[0][0]
    }
    /// 请参阅 [AffineTransform::new] 以了解该值在仿射变换中的作用。
    pub fn b(&self) -> T {
        self.0[0][1]
    }
    /// 请参阅 [AffineTransform::new] 以了解该值在仿射变换中的作用。
    pub fn xoff(&self) -> T {
        self.0[0][2]
    }
    /// 请参阅 [AffineTransform::new] 以了解该值在仿射变换中的作用。
    pub fn d(&self) -> T {
        self.0[1][0]
    }
    /// 请参阅 [AffineTransform::new] 以了解该值在仿射变换中的作用。
    pub fn e(&self) -> T {
        self.0[1][1]
    }
    /// 请参阅 [AffineTransform::new] 以了解该值在仿射变换中的作用。
    pub fn yoff(&self) -> T {
        self.0[1][2]
    }
}

impl<T: CoordNum + Neg> AffineTransform<T> {
    /// 返回给定变换的逆。将变换与其逆组合会得到[单位矩阵](Self::identity)
    #[must_use]
    pub fn inverse(&self) -> Option<Self>
    where
        <T as Neg>::Output: Mul<T>,
        <<T as Neg>::Output as Mul<T>>::Output: ToPrimitive,
    {
        let a = self.0[0][0];
        let b = self.0[0][1];
        let xoff = self.0[0][2];
        let d = self.0[1][0];
        let e = self.0[1][1];
        let yoff = self.0[1][2];

        let determinant = a * e - b * d;

        if determinant == T::zero() {
            return None; // 矩阵不可逆
        }
        let inv_det = T::one() / determinant;

        // 如果 b 或 d 矩阵值的转换失败，则中止
        Some(Self::new(
            e * inv_det,
            T::from(-b * inv_det)?,
            (b * yoff - e * xoff) * inv_det,
            T::from(-d * inv_det)?,
            a * inv_det,
            (d * xoff - a * yoff) * inv_det,
        ))
    }
}

impl<T: CoordNum> fmt::Debug for AffineTransform<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AffineTransform")
            .field("a", &self.0[0][0])
            .field("b", &self.0[0][1])
            .field("xoff", &self.0[0][2])
            .field("d", &self.0[1][0])
            .field("e", &self.0[1][1])
            .field("yoff", &self.0[1][2])
            .finish()
    }
}

impl<T: CoordNum> From<[T; 6]> for AffineTransform<T> {
    fn from(arr: [T; 6]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3], arr[4], arr[5])
    }
}

impl<T: CoordNum> From<(T, T, T, T, T, T)> for AffineTransform<T> {
    fn from(tup: (T, T, T, T, T, T)) -> Self {
        Self::new(tup.0, tup.1, tup.2, tup.3, tup.4, tup.5)
    }
}

impl<U: CoordFloat> AffineTransform<U> {
    /// **创建**旋转的仿射变换，使用任意点作为中心。
    ///
    /// 请注意，此操作仅适用于浮点坐标的几何体。
    ///
    /// `angle` 以**度数**给出。
    ///
    /// 矩阵（角度表示为 theta）为：
    /// ```ignore
    /// [[cos_theta, -sin_theta, xoff],
    /// [sin_theta, cos_theta, yoff],
    /// [0, 0, 1]]
    ///
    /// xoff = origin.x - (origin.x * cos(theta)) + (origin.y * sin(theta))
    /// yoff = origin.y - (origin.x * sin(theta)) + (origin.y * cos(theta))
    /// ```
    pub fn rotate(degrees: U, origin: impl Into<Coord<U>>) -> Self {
        let (sin_theta, cos_theta) = degrees.to_radians().sin_cos();
        let (x0, y0) = origin.into().x_y();
        let xoff = x0 - (x0 * cos_theta) + (y0 * sin_theta);
        let yoff = y0 - (x0 * sin_theta) - (y0 * cos_theta);
        Self::new(cos_theta, -sin_theta, xoff, sin_theta, cos_theta, yoff)
    }

    /// **添加**旋转的仿射变换，使用任意点作为中心。
    ///
    /// 请注意，此操作仅适用于浮点坐标的几何体。
    ///
    /// `angle` 以**度数**给出。
    ///
    /// 这是一个**累积**操作；新变换*添加*到现有变换中。
    #[must_use]
    pub fn rotated(mut self, angle: U, origin: impl Into<Coord<U>>) -> Self {
        self.0 = self.compose(&Self::rotate(angle, origin)).0;
        self
    }

    /// **创建**倾斜的仿射变换。
    ///
    /// 请注意，此操作仅适用于浮点坐标的几何体。
    ///
    /// 将几何体沿 x(`xs`) 和 y(`ys`) 维度按照一定角度剪切。
    /// 原点通常是几何图形的2D边界框中心，但可以指定任何坐标。角度以**度数**给出。
    /// 矩阵为：
    /// ```ignore
    /// [[1, tan(x), xoff],
    /// [tan(y), 1, yoff],
    /// [0, 0, 1]]
    ///
    /// xoff = -origin.y * tan(xs)
    /// yoff = -origin.x * tan(ys)
    /// ```
    pub fn skew(xs: U, ys: U, origin: impl Into<Coord<U>>) -> Self {
        let Coord { x: x0, y: y0 } = origin.into();
        let mut tanx = xs.to_radians().tan();
        let mut tany = ys.to_radians().tan();
        // 这些检查借鉴自 Shapely 的实现 - 可能不必要
        if tanx.abs() < U::from::<f64>(2.5e-16).unwrap() {
            tanx = U::zero();
        }
        if tany.abs() < U::from::<f64>(2.5e-16).unwrap() {
            tany = U::zero();
        }
        let xoff = -y0 * tanx;
        let yoff = -x0 * tany;
        Self::new(U::one(), tanx, xoff, tany, U::one(), yoff)
    }

    /// **添加**倾斜的仿射变换。
    ///
    /// 请注意，此操作仅适用于浮点坐标的几何体。
    ///
    /// 将几何体沿 x(`xs`) 和 y(`ys`) 维度按照一定角度剪切。
    /// 原点通常是几何图形的2D边界框中心，但可以指定任何坐标。角度以**度数**给出。
    ///
    /// 这是一个**累积**操作；新变换*添加*到现有变换中。
    #[must_use]
    pub fn skewed(mut self, xs: U, ys: U, origin: impl Into<Coord<U>>) -> Self {
        self.0 = self.compose(&Self::skew(xs, ys, origin)).0;
        self
    }
}

#[cfg(test)]
mod tests {
    use approx::{AbsDiffEq, RelativeEq};

    impl<T> RelativeEq for AffineTransform<T>
    where
        T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
    {
        #[inline]
        fn default_max_relative() -> Self::Epsilon {
            T::default_max_relative()
        }

        /// 在相对极限内的相等断言。
        ///
        /// # 示例
        ///
        /// ```
        /// use geo_types::AffineTransform;
        /// use geo_types::point;
        ///
        /// let a = AffineTransform::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        /// let b = AffineTransform::new(1.01, 2.02, 3.03, 4.04, 5.05, 6.06);
        ///
        /// approx::assert_relative_eq!(a, b, max_relative=0.1)
        /// approx::assert_relative_ne!(a, b, max_relative=0.055)
        /// ```
        #[inline]
        fn relative_eq(
            &self,
            other: &Self,
            epsilon: Self::Epsilon,
            max_relative: Self::Epsilon,
        ) -> bool {
            let mut mp_zipper = self.0.iter().flatten().zip(other.0.iter().flatten());
            mp_zipper.all(|(lhs, rhs)| lhs.relative_eq(rhs, epsilon, max_relative))
        }
    }

    impl<T> AbsDiffEq for AffineTransform<T>
    where
        T: AbsDiffEq<Epsilon = T> + CoordNum,
        T::Epsilon: Copy,
    {
        type Epsilon = T;

        #[inline]
        fn default_epsilon() -> Self::Epsilon {
            T::default_epsilon()
        }

        /// 具有绝对限制的相等断言。
        ///
        /// # 示例
        ///
        /// ```
        /// use geo_types::MultiPoint;
        /// use geo_types::point;
        ///
        /// let a = AffineTransform::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        /// let b = AffineTransform::new(1.01, 2.02, 3.03, 4.04, 5.05, 6.06);
        ///
        /// approx::abs_diff_eq!(a, b, epsilon=0.1)
        /// approx::abs_diff_ne!(a, b, epsilon=0.055)
        /// ```
        #[inline]
        fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
            let mut mp_zipper = self.0.iter().flatten().zip(other.0.iter().flatten());
            mp_zipper.all(|(lhs, rhs)| lhs.abs_diff_eq(rhs, epsilon))
        }
    }

    use super::*;
    use crate::{wkt, Point};

    // 给定一个形状的矩阵
    // [[a, b, xoff],
    // [d, e, yoff],
    // [0, 0, 1]]
    #[test]
    fn matrix_multiply() {
        let a = AffineTransform::new(1, 2, 5, 3, 4, 6);
        let b = AffineTransform::new(7, 8, 11, 9, 10, 12);
        let composed = a.compose(&b);
        assert_eq!(composed.0[0][0], 31);
        assert_eq!(composed.0[0][1], 46);
        assert_eq!(composed.0[0][2], 94);
        assert_eq!(composed.0[1][0], 39);
        assert_eq!(composed.0[1][1], 58);
        assert_eq!(composed.0[1][2], 117);
    }
    #[test]
    fn test_transform_composition() {
        let p0 = Point::new(0.0f64, 0.0);
        // 缩放一次
        let mut scale_a = AffineTransform::default().scaled(2.0, 2.0, p0);
        // 旋转
        scale_a = scale_a.rotated(45.0, p0);
        // 旋转回去
        scale_a = scale_a.rotated(-45.0, p0);
        // 再次放大，倍增
        scale_a = scale_a.scaled(2.0, 2.0, p0);
        // 缩放一次
        let scale_b = AffineTransform::default().scaled(2.0, 2.0, p0);
        // 缩放一次，但等于2 + 2
        let scale_c = AffineTransform::default().scaled(4.0, 4.0, p0);
        assert_ne!(&scale_a.0, &scale_b.0);
        assert_relative_eq!(&scale_a, &scale_c);
    }

    #[test]
    fn affine_transformed() {
        let transform = AffineTransform::translate(1.0, 1.0).scaled(2.0, 2.0, (0.0, 0.0));
        let mut poly = wkt! { POLYGON((0.0 0.0,0.0 2.0,1.0 2.0)) };
        poly.affine_transform_mut(&transform);

        let expected = wkt! { POLYGON((2.0 2.0,2.0 6.0,4.0 6.0)) };
        assert_eq!(expected, poly);
    }
    #[test]
    fn affine_transformed_inverse() {
        let transform = AffineTransform::translate(1.0, 1.0).scaled(2.0, 2.0, (0.0, 0.0));
        let tinv = transform.inverse().unwrap();
        let identity = transform.compose(&tinv);
        // 测试其实只需要这个，但让我们确保一下
        assert!(identity.is_identity());

        let mut poly = wkt! { POLYGON((0.0 0.0,0.0 2.0,1.0 2.0)) };
        let expected = poly.clone();
        poly.affine_transform_mut(&identity);
        assert_eq!(expected, poly);
    }
    #[test]
    fn test_affine_transform_getters() {
        let transform = AffineTransform::new(10.0, 0.0, 400_000.0, 0.0, -10.0, 500_000.0);
        assert_eq!(transform.a(), 10.0);
        assert_eq!(transform.b(), 0.0);
        assert_eq!(transform.xoff(), 400_000.0);
        assert_eq!(transform.d(), 0.0);
        assert_eq!(transform.e(), -10.0);
        assert_eq!(transform.yoff(), 500_000.0);
    }
    #[test]
    fn test_compose() {
        let point = Point::new(1., 0.);

        let translate = AffineTransform::translate(1., 0.);
        let scale = AffineTransform::scale(4., 1., [0., 0.]);
        let composed = translate.compose(&scale);

        assert_eq!(point.affine_transform(&translate), Point::new(2., 0.));
        assert_eq!(point.affine_transform(&scale), Point::new(4., 0.));
        assert_eq!(
            point.affine_transform(&translate).affine_transform(&scale),
            Point::new(8., 0.)
        );

        assert_eq!(point.affine_transform(&composed), Point::new(8., 0.));
    }
}
