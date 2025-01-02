use super::kernels::*;
use crate::coords_iter::CoordsIter;
use crate::utils::EitherIter;
use crate::{CoordNum, GeoFloat, GeoNum, LineString, Point};
use geo_types::{PointsIter, Triangle};
use std::iter::Rev;

/// 迭代 `Point` 列表
#[allow(missing_debug_implementations)]
pub struct Points<'a, T>(pub(crate) EitherIter<PointsIter<'a, T>, Rev<PointsIter<'a, T>>>)
where
    T: CoordNum + 'a;

impl<T> Iterator for Points<'_, T>
where
    T: CoordNum,
{
    type Item = Point<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T> ExactSizeIterator for Points<'_, T>
where
    T: CoordNum,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// 线字符串的绕行顺序，顺时针或逆时针
#[derive(PartialEq, Clone, Debug, Eq, Copy)]
pub enum WindingOrder {
    Clockwise,        // 顺时针
    CounterClockwise, // 逆时针
}

impl WindingOrder {
    #[allow(dead_code)]
    pub(crate) fn inverse(&self) -> Self {
        match self {
            WindingOrder::Clockwise => WindingOrder::CounterClockwise,
            WindingOrder::CounterClockwise => WindingOrder::Clockwise,
        }
    }
}

/// 确定并操作 [`LineString`] 的绕行顺序。
/// 此功能和我们的实现基于 [CGAL's Polygon_2::orientation].
///
/// [CGAL's Polygon_2::orientation]: //doc.cgal.org/latest/Polygon/classCGAL_1_1Polygon__2.html#a4ce8b4b8395406243ac16c2a120ffc15
pub trait Winding {
    type Scalar: CoordNum;

    /// 返回此对象的绕行顺序，如果包含至少三个不同坐标，则返回相应绕行顺序，反之返回 `None`。
    fn winding_order(&self) -> Option<WindingOrder>;

    /// 若绕行顺序为顺时针，则返回 true
    fn is_cw(&self) -> bool {
        self.winding_order() == Some(WindingOrder::Clockwise)
    }

    /// 若绕行顺序为逆时针，则返回 true
    fn is_ccw(&self) -> bool {
        self.winding_order() == Some(WindingOrder::CounterClockwise)
    }

    /// 按顺时针顺序迭代点
    ///
    /// 对象不变，点返回的顺序或逆序，使得结果顺序看起来为顺时针
    fn points_cw(&self) -> Points<Self::Scalar>;

    /// 按逆时针顺序迭代点
    ///
    /// 对象不变，点返回的顺序或逆序，使得结果顺序看起来为逆时针
    fn points_ccw(&self) -> Points<Self::Scalar>;

    /// 改变此对象的点顺序为顺时针绕行顺序
    fn make_cw_winding(&mut self);

    /// 改变此线的点顺序为逆时针绕行顺序
    fn make_ccw_winding(&mut self);

    /// 返回此对象的克隆，以指定的绕行顺序
    fn clone_to_winding_order(&self, winding_order: WindingOrder) -> Self
    where
        Self: Sized + Clone,
    {
        let mut new: Self = self.clone();
        new.make_winding_order(winding_order);
        new
    }

    /// 改变绕行顺序以设置为指定的绕行顺序
    fn make_winding_order(&mut self, winding_order: WindingOrder) {
        match winding_order {
            WindingOrder::Clockwise => self.make_cw_winding(),
            WindingOrder::CounterClockwise => self.make_ccw_winding(),
        }
    }
}

impl<T, K> Winding for LineString<T>
where
    T: GeoNum<Ker = K>,
    K: Kernel<T>,
{
    type Scalar = T;

    fn winding_order(&self) -> Option<WindingOrder> {
        // 如果 linestring 的坐标数不超过3，它要么未闭合，要么最多两个不同点。无论哪种方式，绕行顺序未指定。
        if self.coords_count() < 4 || !self.is_closed() {
            return None;
        }

        let increment = |x: &mut usize| {
            *x += 1;
            if *x >= self.coords_count() {
                *x = 0;
            }
        };

        let decrement = |x: &mut usize| {
            if *x == 0 {
                *x = self.coords_count() - 1;
            } else {
                *x -= 1;
            }
        };

        use crate::utils::least_index;
        let i = least_index(&self.0);

        let mut next = i;
        increment(&mut next);
        while self.0[next] == self.0[i] {
            if next == i {
                // 我们循环得太多了。 没有足够唯一坐标来计算方向。
                return None;
            }
            increment(&mut next);
        }

        let mut prev = i;
        decrement(&mut prev);
        while self.0[prev] == self.0[i] {
            // 注意：我们不需要检查 prev == i，因为前一个循环成功，所以列表中至少有两个不同的元素
            decrement(&mut prev);
        }

        match K::orient2d(self.0[prev], self.0[i], self.0[next]) {
            Orientation::CounterClockwise => Some(WindingOrder::CounterClockwise),
            Orientation::Clockwise => Some(WindingOrder::Clockwise),
            _ => None,
        }
    }

    /// 按顺时针顺序迭代点
    ///
    /// Linestring 不变，点返回的顺序或逆序，使得结果顺序看起来为顺时针
    fn points_cw(&self) -> Points<Self::Scalar> {
        match self.winding_order() {
            Some(WindingOrder::CounterClockwise) => Points(EitherIter::B(self.points().rev())),
            _ => Points(EitherIter::A(self.points())),
        }
    }

    /// 按逆时针顺序迭代点
    ///
    /// Linestring 不变，点返回的顺序或逆序，使得结果顺序看起来为逆时针
    fn points_ccw(&self) -> Points<Self::Scalar> {
        match self.winding_order() {
            Some(WindingOrder::Clockwise) => Points(EitherIter::B(self.points().rev())),
            _ => Points(EitherIter::A(self.points())),
        }
    }

    /// 改变此线的点顺序为顺时针绕行顺序
    fn make_cw_winding(&mut self) {
        if let Some(WindingOrder::CounterClockwise) = self.winding_order() {
            self.0.reverse();
        }
    }

    /// 改变此线的点顺序为逆时针绕行顺序
    fn make_ccw_winding(&mut self) {
        if let Some(WindingOrder::Clockwise) = self.winding_order() {
            self.0.reverse();
        }
    }
}

// 此函数可能通过小重构特质实现转换为特质实现，但不在本次 PR 的范围内添加.
/// 用于寻找三角形绕行顺序的特殊算法
pub fn triangle_winding_order<T: GeoFloat>(tri: &Triangle<T>) -> Option<WindingOrder> {
    let [a, b, c] = tri.to_array();
    let ab = b - a;
    let ac = c - a;

    let cross_prod = ab.x * ac.y - ab.y * ac.x;

    match cross_prod.total_cmp(&T::zero()) {
        std::cmp::Ordering::Less => Some(WindingOrder::Clockwise),
        std::cmp::Ordering::Equal => None,
        std::cmp::Ordering::Greater => Some(WindingOrder::CounterClockwise),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Point;

    #[test]
    fn robust_winding_float() {
        // 三个点构成一个三角形
        let a = Point::new(0., 0.);
        let b = Point::new(2., 0.);
        let c = Point::new(1., 2.);

        // 验证未闭合的线字符串返回 None
        let mut ls = LineString::from(vec![a.0, b.0, c.0]);
        assert!(ls.winding_order().is_none());

        ls.0.push(ls.0[0]);
        assert_eq!(ls.winding_order(), Some(WindingOrder::CounterClockwise));

        ls.make_cw_winding();
        assert_eq!(ls.winding_order(), Some(WindingOrder::Clockwise));
    }

    #[test]
    fn robust_winding_integer() {
        // 三个点构成一个三角形
        let a = Point::new(0i64, 0);
        let b = Point::new(2, 0);
        let c = Point::new(1, 2);

        // 验证未闭合的线字符串返回 None
        let mut ls = LineString::from(vec![a.0, b.0, c.0]);
        assert!(ls.winding_order().is_none());

        ls.0.push(ls.0[0]);
        assert!(ls.is_ccw());

        let ccw_ls: Vec<_> = ls.points_ccw().collect();

        ls.make_cw_winding();
        assert!(ls.is_cw());

        assert_eq!(&ls.points_ccw().collect::<Vec<_>>(), &ccw_ls,);
    }
}
