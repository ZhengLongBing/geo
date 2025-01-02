use crate::algorithm::Contains;

/// 测试一个几何图形是否完全在另一个几何图形内。
///
/// 换句话说，(Self, Rhs) 的 [DE-9IM] 交集矩阵是 `[T*F**F***]`。
///
/// # 示例
///
/// ```
/// use geo::{point, line_string};
/// use geo::algorithm::Within;
///
/// let line_string = line_string![(x: 0.0, y: 0.0), (x: 2.0, y: 4.0)];
///
/// assert!(point!(x: 1.0, y: 2.0).is_within(&line_string));
///
/// // 注意：仅位于另一个几何图形*边界*上的几何图形不被认为在该几何图形的内部。
/// // 更多信息参见 [`Relate`]。
/// assert!(! point!(x: 0.0, y: 0.0).is_within(&line_string));
/// ```
///
/// `Within` 等价于参数交换后的 [`Contains`]。
///
/// ```
/// use geo::{point, line_string};
/// use geo::algorithm::{Contains, Within};
///
/// let line_string = line_string![(x: 0.0, y: 0.0), (x: 2.0, y: 4.0)];
/// let point = point!(x: 1.0, y: 2.0);
///
/// // 这两种比较是完全等价的
/// assert!(point.is_within(&line_string));
/// assert!(line_string.contains(&point));
/// ```
///
/// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM
pub trait Within<Other> {
    fn is_within(&self, b: &Other) -> bool; // 判断当前几何图形是否在另一个几何图形内
}

impl<G1, G2> Within<G2> for G1
where
    G2: Contains<G1>,
{
    fn is_within(&self, b: &G2) -> bool {
        b.contains(self) // 使用Contains trait来实现is_within方法
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{point, Rect};

    #[test]
    fn basic() {
        // 测试点和多边形之间的包含关系
        let a = point!(x: 1.0, y: 2.0);
        let b = Rect::new((0.0, 0.0), (3.0, 3.0)).to_polygon();
        assert!(a.is_within(&b));
    }
}
