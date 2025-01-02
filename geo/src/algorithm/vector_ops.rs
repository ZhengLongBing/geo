//! 本模块定义了 [Vector2DOps] 特性，并为 [Coord] 结构体实现。

use crate::{Coord, CoordFloat, CoordNum};

/// 为实现 CoordFloat 的二维坐标类型定义向量运算
///
/// 此特性用于 geo crate 的内部，作为一种方法，将使用于其他算法中的各种手工线性代数运算汇总在一起，并附加到不同的结构体上。
pub trait Vector2DOps<Rhs = Self>
where
    Self: Sized,
{
    type Scalar: CoordNum;

    /// 该坐标到原点的欧几里得距离
    ///
    /// `sqrt(x² + y²)`
    ///
    fn magnitude(self) -> Self::Scalar;

    /// 该坐标到原点的平方距离。
    /// （避免在不需要时进行平方根计算）
    ///
    /// `x² + y²`
    ///
    fn magnitude_squared(self) -> Self::Scalar;

    /// 将此坐标绕原点顺时针旋转90度。
    ///
    /// `a.left() => (-a.y, a.x)`
    ///
    /// 假设坐标系中正 `y` 向上，正 `x` 向右。描述的旋转方向与 [crate::algorithm::rotate::Rotate] 文档一致。
    fn left(self) -> Self;

    /// 将此坐标绕原点逆时针旋转90度。
    ///
    /// `a.right() => (a.y, -a.x)`
    ///
    /// 假设坐标系中正 `y` 向上，正 `x` 向右。描述的旋转方向与 [crate::algorithm::rotate::Rotate] 文档一致。
    fn right(self) -> Self;

    /// 坐标分量的内积
    ///
    /// `a · b = a.x * b.x + a.y * b.y`
    ///
    fn dot_product(self, other: Rhs) -> Self::Scalar;

    /// 计算两个向量之间的 `楔积`。
    ///
    /// `a ∧ b = a.x * b.y - a.y * b.x`
    ///
    /// 也称为：
    ///
    ///  - `外积`
    ///    - 因为楔积来自'Exterior Algebra'
    ///  - `垂直乘积`
    ///    - 因为它相当于 `a.dot(b.right())`
    ///  - `二维叉积`
    ///    - 因为它等同于假设 `z` 坐标为零的常规三维叉积的有符号大小
    ///  - `行列式`
    ///    - 因为它等效于由列向量输入组成的 2x2 矩阵的`行列式`。
    ///
    /// ## 示例
    ///
    /// 以下列出了一些 geo 中可以使用此功能的示例：
    ///
    /// 1. [geo_types::Point::cross_prod()] 已经在 [geo_types::Point] 上定义...但似乎是对3个点的某种其他运算？
    /// 2. [geo_types::Line] 结构体也有一个 [geo_types::Line::determinant()] 函数，对应于 `line.start.wedge_product(line.end)`
    /// 3. [crate::algorithm::Kernel::orient2d()] 特性的默认实现使用交叉积来计算方向。 它返回一个枚举，而不是线段交点所需的数值。
    ///
    /// ## 属性
    ///
    /// - 叉积的绝对值是操作数形成的平行四边形的面积
    /// - 反交换：如果交换操作数，输出符号将反转
    /// - 如果操作数与原点共线，值为零
    /// - 符号可以用来检查操作数是否以原点为中心顺时针排列，或者换句话说：
    ///   "a 是否在以原点为起点、b 为终点的线的左侧"？
    ///   - 如果您用于此用途，请使用 [crate::algorithm::Kernel::orient2d()]，因为这更明确并具有 `RobustKernel` 选项以提高精度。
    fn wedge_product(self, other: Rhs) -> Self::Scalar;

    /// 尝试找到与此向量方向相同的单位长度向量。
    ///
    /// 如果结果不是有限值，则返回 `None`。这可能发生在
    ///
    /// — 向量非常小（或零长度），`.magnitude()` 计算四舍五入为 `0.0`
    /// — 向量非常大并且 `.magnitude()` 四舍五入为或“溢出”为 `f64::INFINITY`
    /// — x 或 y 为 `f64::NAN` 或 `f64::INFINITY`
    fn try_normalize(self) -> Option<Self>;

    /// 如果 x 和 y 组件都是有限的，则返回 true
    // 注释是为了禁用错误的 clippy lint； 使用 &self 不好，因为 Coord 是 Copy
    #[allow(clippy::wrong_self_convention)]
    fn is_finite(self) -> bool;
}

impl<T> Vector2DOps for Coord<T>
where
    T: CoordFloat,
{
    type Scalar = T;

    fn wedge_product(self, other: Coord<T>) -> Self::Scalar {
        self.x * other.y - self.y * other.x
    }

    fn dot_product(self, other: Self) -> Self::Scalar {
        self.x * other.x + self.y * other.y
    }

    fn magnitude(self) -> Self::Scalar {
        // 注意使用 cmath::hypot 避免“不必要的溢出和下溢”
        // 这也增加了 `.try_normalize()` 可以工作的值范围
        Self::Scalar::hypot(self.x, self.y)
    }

    fn magnitude_squared(self) -> Self::Scalar {
        self.x * self.x + self.y * self.y
    }

    fn left(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    fn right(self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
        }
    }

    fn try_normalize(self) -> Option<Self> {
        let magnitude = self.magnitude();
        let result = self / magnitude;
        // 结果和幅度都必须有限
        // 否则非常大的向量将幅度溢出为 Infinity，
        // 然后在除法后，结果将是 coord!{x:0.0,y:0.0}
        // 注意我们不需要检查幅度是否为零，因为除法后会导致结果不是有限值或 NaN 无论如何。
        if result.is_finite() && magnitude.is_finite() {
            Some(result)
        } else {
            None
        }
    }

    fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

#[cfg(test)]
mod test {
    use super::Vector2DOps;
    use crate::coord;

    #[test]
    fn test_cross_product() {
        // 垂直单位长度
        let a = coord! { x: 1f64, y: 0f64 };
        let b = coord! { x: 0f64, y: 1f64 };

        // 期望为平行四边形的面积
        assert_eq!(a.wedge_product(b), 1f64);
        // 期待交换将导致负值
        assert_eq!(b.wedge_product(a), -1f64);

        // 添加偏差；期望结果应相同
        let a = coord! { x: 1f64, y: 0f64 };
        let b = coord! { x: 1f64, y: 1f64 };

        // 期望为平行四边形的面积
        assert_eq!(a.wedge_product(b), 1f64);
        // 期待交换将导致负值
        assert_eq!(b.wedge_product(a), -1f64);

        // 使共线；期望为零
        let a = coord! { x: 2f64, y: 2f64 };
        let b = coord! { x: 1f64, y: 1f64 };
        assert_eq!(a.wedge_product(b), 0f64);
    }

    #[test]
    fn test_dot_product() {
        // 垂直单位长度
        let a = coord! { x: 1f64, y: 0f64 };
        let b = coord! { x: 0f64, y: 1f64 };
        // 垂直时期待为零
        assert_eq!(a.dot_product(b), 0f64);

        // 平行，同方向
        let a = coord! { x: 1f64, y: 0f64 };
        let b = coord! { x: 2f64, y: 0f64 };
        // 期望与幅度之积相同
        assert_eq!(a.dot_product(b), 2f64);
        // 期待交换会有同样的结果
        assert_eq!(b.dot_product(a), 2f64);

        // 平行，相反方向
        let a = coord! { x: 3f64, y: 4f64 };
        let b = coord! { x: -3f64, y: -4f64 };
        // 期望负值的幅度之积
        assert_eq!(a.dot_product(b), -25f64);
        // 期待交换会有同样的结果
        assert_eq!(b.dot_product(a), -25f64);
    }

    #[test]
    fn test_magnitude() {
        let a = coord! { x: 1f64, y: 0f64 };
        assert_eq!(a.magnitude(), 1f64);

        let a = coord! { x: 0f64, y: 0f64 };
        assert_eq!(a.magnitude(), 0f64);

        let a = coord! { x: -3f64, y: 4f64 };
        assert_eq!(a.magnitude(), 5f64);
    }

    #[test]
    fn test_magnitude_squared() {
        let a = coord! { x: 1f64, y: 0f64 };
        assert_eq!(a.magnitude_squared(), 1f64);

        let a = coord! { x: 0f64, y: 0f64 };
        assert_eq!(a.magnitude_squared(), 0f64);

        let a = coord! { x: -3f64, y: 4f64 };
        assert_eq!(a.magnitude_squared(), 25f64);
    }

    #[test]
    fn test_left_right() {
        let a = coord! { x: 1f64, y: 0f64 };
        let a_left = coord! { x: 0f64, y: 1f64 };
        let a_right = coord! { x: 0f64, y: -1f64 };

        assert_eq!(a.left(), a_left);
        assert_eq!(a.right(), a_right);
        assert_eq!(a.left(), -a.right());
    }

    #[test]
    fn test_left_right_match_rotate() {
        use crate::algorithm::rotate::Rotate;
        use crate::Point;
        // 此测试的目的是确认文档中的措辞是一致的。

        // 当用户位于 y 轴翻转的坐标系中时
        // （例如 HTML 画布中的屏幕坐标），旋转方向将不同于文档中描述的方向。

        // Rotate 特性的文档说明：'正角度为逆时针方向，负角度为顺时针方向'

        let counter_clockwise_rotation_degrees = 90.0;
        let clockwise_rotation_degrees = -counter_clockwise_rotation_degrees;

        let a: Point = coord! { x: 1.0, y: 0.0 }.into();
        let origin: Point = coord! { x: 0.0, y: 0.0 }.into();

        // 左为逆时针
        assert_relative_eq!(
            Point::from(a.0.left()),
            a.rotate_around_point(counter_clockwise_rotation_degrees, origin),
        );
        // 右为顺时针
        assert_relative_eq!(
            Point::from(a.0.right()),
            a.rotate_around_point(clockwise_rotation_degrees, origin),
        );
    }

    #[test]
    fn test_try_normalize() {
        // 已经标准化
        let a = coord! {
            x: 1.0,
            y: 0.0
        };
        assert_relative_eq!(a.try_normalize().unwrap(), a);

        // 已经标准化
        let a = coord! {
            x: 1.0 / f64::sqrt(2.0),
            y: -1.0 / f64::sqrt(2.0)
        };
        assert_relative_eq!(a.try_normalize().unwrap(), a);

        // 非平凡示例
        let a = coord! { x: -10.0, y: 8.0 };
        assert_relative_eq!(
            a.try_normalize().unwrap(),
            coord! { x: -10.0, y: 8.0 } / f64::sqrt(10.0 * 10.0 + 8.0 * 8.0)
        );
    }

    #[test]
    /// 测试以前在切换到 cmath::hypot 之前返回 None 的边缘情况
    fn test_try_normalize_edge_cases_1() {
        use float_next_after::NextAfter;
        // 非常小的输入仍然可以返回值，感谢 cmath::hypot
        let a = coord! {
            x: 0.0,
            y: 1e-301_f64
        };
        assert_eq!(
            a.try_normalize(),
            Some(coord! {
                x: 0.0,
                y: 1.0,
            })
        );

        // 一个大的向量，其中 try_normalize 返回 Some
        // 因为幅度是 f64::MAX（刚好到达溢出到 f64::INFINITY 之前）
        let a = coord! {
            x: f64::sqrt(f64::MAX/2.0),
            y: f64::sqrt(f64::MAX/2.0)
        };
        assert_relative_eq!(
            a.try_normalize().unwrap(),
            coord! {
                x: 1.0 / f64::sqrt(2.0),
                y: 1.0 / f64::sqrt(2.0),
            }
        );

        // 一个大的向量，where try_normalize 仍然返回 Some，因为我们正在使用 cmath::hypot
        // 尽管幅度刚好超过 f64::MAX
        let a = coord! {
            x: f64::sqrt(f64::MAX / 2.0),
            y: f64::sqrt(f64::MAX / 2.0).next_after(f64::INFINITY)
        };
        assert_relative_eq!(
            a.try_normalize().unwrap(),
            coord! {
                x: 1.0 / f64::sqrt(2.0),
                y: 1.0 / f64::sqrt(2.0),
            }
        );
    }

    #[test]
    fn test_try_normalize_edge_cases_2() {
        // 以下测试展示了一些导致 try_normalize 返回 None 的浮点数边缘情况。

        // 零向量 - 归一化返回 None
        let a = coord! { x: 0.0, y: 0.0 };
        assert_eq!(a.try_normalize(), None);

        // 如果其中一个分量为 NaN，则 try_normalize 返回 None
        let a = coord! { x: f64::NAN, y: 0.0 };
        assert_eq!(a.try_normalize(), None);

        // 如果其中一个分量为 Infinite，则 try_normalize 返回 None
        let a = coord! { x: f64::INFINITY, y: 0.0 };
        assert_eq!(a.try_normalize(), None);
    }
}
