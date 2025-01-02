use crate::{CoordNum, LineString};

use alloc::vec;
use alloc::vec::Vec;
#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};
use core::iter::FromIterator;
#[cfg(feature = "multithreading")]
use rayon::prelude::*;

/// [`LineString`](line_string/struct.LineString.html)的集合。
/// 可以从`LineString`的`Vec`或产生`LineString`的迭代器创建。
/// 迭代此对象会产生组成的`LineString`。
///
/// # 语义
///
/// `MultiLineString`的_边界_通过应用"模2"并集规则获得：
/// 如果一个`Point`在`MultiLineString`的奇数个元素的边界上，
/// 则它在`MultiLineString`的边界上。
///
/// `MultiLineString`的_内部_是组成`LineString`的内部和边界的并集，
/// _除了_上面定义的边界。换句话说，它是边界与组成部分的内部和边界的并集的差集。
///
/// 当且仅当所有元素都是简单的，并且任意两个元素之间的唯一交点
/// 发生在两个元素边界上的`Point`处时，`MultiLineString`是_简单的_。
/// 如果所有元素都是闭合的，则`MultiLineString`是_闭合的_。
/// 闭合`MultiLineString`的边界始终为空。
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiLineString<T: CoordNum = f64>(pub Vec<LineString<T>>);

impl<T: CoordNum> MultiLineString<T> {
    /// 从原始内容值实例化Self
    pub fn new(value: Vec<LineString<T>>) -> Self {
        Self(value)
    }

    /// 如果MultiLineString为空或其所有LineString都是闭合的，则返回true - 参见
    /// [`LineString::is_closed`]。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::{MultiLineString, LineString, line_string};
    ///
    /// let open_line_string: LineString<f32> = line_string![(x: 0., y: 0.), (x: 5., y: 0.)];
    /// assert!(!MultiLineString::new(vec![open_line_string.clone()]).is_closed());
    ///
    /// let closed_line_string: LineString<f32> = line_string![(x: 0., y: 0.), (x: 5., y: 0.), (x: 0., y: 0.)];
    /// assert!(MultiLineString::new(vec![closed_line_string.clone()]).is_closed());
    ///
    /// // 如果*任何*LineString未闭合，则MultiLineString不闭合
    /// assert!(!MultiLineString::new(vec![open_line_string, closed_line_string]).is_closed());
    ///
    /// // 空的MultiLineString是闭合的
    /// assert!(MultiLineString::<f32>::new(vec![]).is_closed());
    /// ```
    pub fn is_closed(&self) -> bool {
        // 注意：与JTS等不同，我们认为空的MultiLineString是闭合的。
        self.iter().all(LineString::is_closed)
    }
}

impl<T: CoordNum, ILS: Into<LineString<T>>> From<ILS> for MultiLineString<T> {
    fn from(ls: ILS) -> Self {
        Self(vec![ls.into()])
    }
}

impl<T: CoordNum, ILS: Into<LineString<T>>> FromIterator<ILS> for MultiLineString<T> {
    fn from_iter<I: IntoIterator<Item = ILS>>(iter: I) -> Self {
        Self(iter.into_iter().map(|ls| ls.into()).collect())
    }
}

impl<T: CoordNum> IntoIterator for MultiLineString<T> {
    type Item = LineString<T>;
    type IntoIter = ::alloc::vec::IntoIter<LineString<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a MultiLineString<T> {
    type Item = &'a LineString<T>;
    type IntoIter = ::alloc::slice::Iter<'a, LineString<T>>;

    fn into_iter(self) -> Self::IntoIter {
        (self.0).iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a mut MultiLineString<T> {
    type Item = &'a mut LineString<T>;
    type IntoIter = ::alloc::slice::IterMut<'a, LineString<T>>;

    fn into_iter(self) -> Self::IntoIter {
        (self.0).iter_mut()
    }
}

impl<T: CoordNum> MultiLineString<T> {
    pub fn iter(&self) -> impl Iterator<Item = &LineString<T>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut LineString<T>> {
        self.0.iter_mut()
    }
}

#[cfg(feature = "multithreading")]
impl<T: CoordNum + Send> IntoParallelIterator for MultiLineString<T> {
    type Item = LineString<T>;
    type Iter = rayon::vec::IntoIter<LineString<T>>;

    fn into_par_iter(self) -> Self::Iter {
        self.0.into_par_iter()
    }
}

#[cfg(feature = "multithreading")]
impl<'a, T: CoordNum + Sync> IntoParallelIterator for &'a MultiLineString<T> {
    type Item = &'a LineString<T>;
    type Iter = rayon::slice::Iter<'a, LineString<T>>;

    fn into_par_iter(self) -> Self::Iter {
        self.0.par_iter()
    }
}

#[cfg(feature = "multithreading")]
impl<'a, T: CoordNum + Send + Sync> IntoParallelIterator for &'a mut MultiLineString<T> {
    type Item = &'a mut LineString<T>;
    type Iter = rayon::slice::IterMut<'a, LineString<T>>;

    fn into_par_iter(self) -> Self::Iter {
        self.0.par_iter_mut()
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for MultiLineString<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    /// 在相对限制内的相等性断言。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::{MultiLineString, line_string};
    ///
    /// let a = MultiLineString::new(vec![line_string![(x: 0., y: 0.), (x: 10., y: 10.)]]);
    /// let b = MultiLineString::new(vec![line_string![(x: 0., y: 0.), (x: 10.01, y: 10.)]]);
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
        if self.0.len() != other.0.len() {
            return false;
        }

        let mut mp_zipper = self.iter().zip(other.iter());
        mp_zipper.all(|(lhs, rhs)| lhs.relative_eq(rhs, epsilon, max_relative))
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> AbsDiffEq for MultiLineString<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    T::Epsilon: Copy,
{
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    /// 带有绝对限制的相等性断言。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::{MultiLineString, line_string};
    ///
    /// let a = MultiLineString::new(vec![line_string![(x: 0., y: 0.), (x: 10., y: 10.)]]);
    /// let b = MultiLineString::new(vec![line_string![(x: 0., y: 0.), (x: 10.01, y: 10.)]]);
    ///
    /// approx::abs_diff_eq!(a, b, epsilon=0.1);
    /// approx::abs_diff_ne!(a, b, epsilon=0.001);
    /// ```
    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        let mut mp_zipper = self.into_iter().zip(other);
        mp_zipper.all(|(lhs, rhs)| lhs.abs_diff_eq(rhs, epsilon))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{line_string, wkt};

    #[cfg(feature = "multithreading")]
    #[test]
    fn test_multithreading_linestring() {
        let multi: MultiLineString<i32> = wkt! {
            MULTILINESTRING((0 0,2 0,1 2,0 0), (10 10,12 10,11 12,10 10))
        };
        let mut multimut: MultiLineString<i32> = wkt! {
            MULTILINESTRING((0 0,2 0,1 2,0 0), (10 10,12 10,11 12,10 10))
        };
        multi.par_iter().for_each(|_p| ());
        multimut.par_iter_mut().for_each(|_p| ());
        let _ = &multi.into_par_iter().for_each(|_p| ());
        let _ = &mut multimut.par_iter_mut().for_each(|_p| ());
    }

    #[test]
    fn test_iter() {
        let multi: MultiLineString<i32> = wkt! {
            MULTILINESTRING((0 0,2 0,1 2,0 0), (10 10,12 10,11 12,10 10))
        };

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(p, &wkt! { LINESTRING(0 0,2 0,1 2,0 0) });
                first = false;
            } else {
                assert_eq!(p, &wkt! { LINESTRING(10 10,12 10,11 12,10 10) });
            }
        }

        // 再次执行以证明`multi`没有被`移动`。
        first = true;
        for p in &multi {
            if first {
                assert_eq!(p, &wkt! { LINESTRING(0 0,2 0,1 2,0 0) });
                first = false;
            } else {
                assert_eq!(p, &wkt! { LINESTRING(10 10,12 10,11 12,10 10) });
            }
        }
    }

    #[test]
    fn test_iter_mut() {
        let mut multi = MultiLineString::new(vec![
            line_string![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)],
            line_string![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)],
        ]);

        for line_string in &mut multi {
            for coord in line_string {
                coord.x += 1;
                coord.y += 1;
            }
        }

        for line_string in multi.iter_mut() {
            for coord in line_string {
                coord.x += 1;
                coord.y += 1;
            }
        }

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &line_string![(x: 2, y: 2), (x: 4, y: 2), (x: 3, y: 4), (x:2, y:2)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &line_string![(x: 12, y: 12), (x: 14, y: 12), (x: 13, y: 14), (x:12, y:12)]
                );
            }
        }
    }
}
