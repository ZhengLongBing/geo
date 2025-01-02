use crate::geometry::{Coord, LineString, Polygon};
use crate::kernels::*;
use crate::GeoNum;

/// 返回几何图形的凸包。凸包总是逆时针方向。
///
/// 此实现使用快速凸包算法，
/// 基于 [Barber, C. Bradford; Dobkin, David P.; Huhdanpaa, Hannu (1996年12月1日)](https://dx.doi.org/10.1145%2F235815.235821)
/// 原始论文链接：<http://www.cs.princeton.edu/~dpd/Papers/BarberDobkinHuhdanpaa.pdf>
///
/// # 示例
///
/// ```
/// use geo::{line_string, polygon};
/// use geo::ConvexHull;
///
/// // 一个L形
/// let poly = polygon![
///     (x: 0.0, y: 0.0),
///     (x: 4.0, y: 0.0),
///     (x: 4.0, y: 1.0),
///     (x: 1.0, y: 1.0),
///     (x: 1.0, y: 4.0),
///     (x: 0.0, y: 4.0),
///     (x: 0.0, y: 0.0),
/// ];
///
/// // 正确的凸包坐标
/// let correct_hull = line_string![
///     (x: 4.0, y: 0.0),
///     (x: 4.0, y: 1.0),
///     (x: 1.0, y: 4.0),
///     (x: 0.0, y: 4.0),
///     (x: 0.0, y: 0.0),
///     (x: 4.0, y: 0.0),
/// ];
///
/// let res = poly.convex_hull();
/// assert_eq!(res.exterior(), &correct_hull);
/// assert_eq!(res.interiors(), &[]);
/// ```
pub trait ConvexHull<'a, T> {
    type Scalar: GeoNum;
    fn convex_hull(&'a self) -> Polygon<Self::Scalar>;
}

use crate::algorithm::CoordsIter;
use crate::utils::lex_cmp;

impl<'a, T, G> ConvexHull<'a, T> for G
where
    T: GeoNum,
    G: CoordsIter<Scalar = T>,
{
    type Scalar = T;

    fn convex_hull(&'a self) -> Polygon<T> {
        let mut exterior: Vec<_> = self.exterior_coords_iter().collect();
        Polygon::new(quick_hull(&mut exterior), vec![])
    }
}

pub mod qhull;
pub use qhull::quick_hull;

pub mod graham;
pub use graham::graham_hull;

// 辅助函数，用于在简单情况下输出凸包：输入最多为 3 个点。它确保输出是逆时针的，并且不会重复点，除非需要。
fn trivial_hull<T>(points: &mut [Coord<T>], include_on_hull: bool) -> LineString<T>
where
    T: GeoNum,
{
    assert!(points.len() < 4);

    // 除非需要包含共线点，否则删除重复点。
    let mut ls: Vec<Coord<T>> = points.to_vec();
    if !include_on_hull {
        ls.sort_unstable_by(lex_cmp);
        if ls.len() == 3 && T::Ker::orient2d(ls[0], ls[1], ls[2]) == Orientation::Collinear {
            ls.remove(1);
        }
    }

    // 一个仅有单个点的线串是无效的。
    if ls.len() == 1 {
        ls.push(ls[0]);
    }

    let mut ls = LineString::new(ls);
    ls.close();

    // 维护逆时针不变性
    use super::winding_order::Winding;
    ls.make_ccw_winding();
    ls
}

/// 凸包操作的工具函数
///
/// 1. 交换 `idx` 处的元素与 `head`（第0个位置）的元素
/// 2. 移除新的 `head` 元素（修改切片）
/// 3. 返回被移除的头元素的可变引用
fn swap_with_first_and_remove<'a, T>(slice: &mut &'a mut [T], idx: usize) -> &'a mut T {
    // 临时用一个空值替换 `slice`
    let tmp = std::mem::take(slice);
    tmp.swap(0, idx);
    let (h, t) = tmp.split_first_mut().unwrap();
    *slice = t;
    h
}

#[cfg(test)]
mod test;
