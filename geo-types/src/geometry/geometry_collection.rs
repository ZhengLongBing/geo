use crate::{CoordNum, Geometry};

use alloc::vec;
use alloc::vec::Vec;
#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};
use core::iter::FromIterator;
use core::ops::{Index, IndexMut};

/// [`Geometry`](enum.Geometry.html) 类型的集合。
///
/// 它可以从一个 Geometries 的 `Vec` 创建，或者从一个产生 Geometries 的迭代器创建。
///
/// 遍历这个对象会产生其组成的 **Geometry
/// 枚举成员**（_不是_ 底层的几何图元），
/// 它支持迭代和索引，以及各种
/// [`MapCoords`](algorithm/map_coords/index.html)
/// 函数，这些函数 _直接_ 应用于底层的几何图元。
///
/// # 示例
/// ## 循环
///
/// ```
/// use std::convert::TryFrom;
/// use geo_types::{Point, point, Geometry, GeometryCollection};
/// let p = point!(x: 1.0, y: 1.0);
/// let pe = Geometry::Point(p);
/// let gc = GeometryCollection::new_from(vec![pe]);
/// for geom in gc {
///     println!("{:?}", Point::try_from(geom).unwrap().x());
/// }
/// ```
/// ## 实现 `iter()`
///
/// ```
/// use std::convert::TryFrom;
/// use geo_types::{Point, point, Geometry, GeometryCollection};
/// let p = point!(x: 1.0, y: 1.0);
/// let pe = Geometry::Point(p);
/// let gc = GeometryCollection::new_from(vec![pe]);
/// gc.iter().for_each(|geom| println!("{:?}", geom));
/// ```
///
/// ## 可变迭代
///
/// ```
/// use std::convert::TryFrom;
/// use geo_types::{Point, point, Geometry, GeometryCollection};
/// let p = point!(x: 1.0, y: 1.0);
/// let pe = Geometry::Point(p);
/// let mut gc = GeometryCollection::new_from(vec![pe]);
/// gc.iter_mut().for_each(|geom| {
///    if let Geometry::Point(p) = geom {
///        p.set_x(0.2);
///    }
/// });
/// let updated = gc[0].clone();
/// assert_eq!(Point::try_from(updated).unwrap().x(), 0.2);
/// ```
///
/// ## 索引
///
/// ```
/// use std::convert::TryFrom;
/// use geo_types::{Point, point, Geometry, GeometryCollection};
/// let p = point!(x: 1.0, y: 1.0);
/// let pe = Geometry::Point(p);
/// let gc = GeometryCollection::new_from(vec![pe]);
/// println!("{:?}", gc[0]);
/// ```
///
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GeometryCollection<T: CoordNum = f64>(pub Vec<Geometry<T>>);

// 手动实现 Default，因为 T 没有 Default 限制
// todo: 考虑将 Default 添加为 CoordNum 要求
impl<T: CoordNum> Default for GeometryCollection<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: CoordNum> GeometryCollection<T> {
    /// 返回一个空的 GeometryCollection
    #[deprecated(
        note = "将在即将发布的版本中被参数化版本替代。请使用 GeometryCollection::default() 代替"
    )]
    pub fn new() -> Self {
        GeometryCollection::default()
    }

    /// 请勿使用！
    /// 这个函数将在即将发布的版本中重命名为 `new`。
    /// 这个函数没有标记为废弃，因为这样做需要对 geo 代码进行大量重构。
    pub fn new_from(value: Vec<Geometry<T>>) -> Self {
        Self(value)
    }

    /// 此 GeometryCollection 中的几何图形数量
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 此 GeometryCollection 是否为空
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// **请勿使用！** 自 0.7.5 版本起已废弃。
///
/// 请使用 `GeometryCollection::from(vec![geom])` 代替。
impl<T: CoordNum, IG: Into<Geometry<T>>> From<IG> for GeometryCollection<T> {
    fn from(x: IG) -> Self {
        Self(vec![x.into()])
    }
}

impl<T: CoordNum, IG: Into<Geometry<T>>> From<Vec<IG>> for GeometryCollection<T> {
    fn from(geoms: Vec<IG>) -> Self {
        let geoms: Vec<Geometry<_>> = geoms.into_iter().map(Into::into).collect();
        Self(geoms)
    }
}

/// 将 Geometries（或可以转换为 Geometry 的对象）收集到 GeometryCollection 中
impl<T: CoordNum, IG: Into<Geometry<T>>> FromIterator<IG> for GeometryCollection<T> {
    fn from_iter<I: IntoIterator<Item = IG>>(iter: I) -> Self {
        Self(iter.into_iter().map(|g| g.into()).collect())
    }
}

impl<T: CoordNum> Index<usize> for GeometryCollection<T> {
    type Output = Geometry<T>;

    fn index(&self, index: usize) -> &Geometry<T> {
        self.0.index(index)
    }
}

impl<T: CoordNum> IndexMut<usize> for GeometryCollection<T> {
    fn index_mut(&mut self, index: usize) -> &mut Geometry<T> {
        self.0.index_mut(index)
    }
}

// 消耗型迭代器的结构辅助
#[derive(Debug)]
pub struct IntoIteratorHelper<T: CoordNum> {
    iter: ::alloc::vec::IntoIter<Geometry<T>>,
}

// 为消耗型迭代器实现 IntoIterator trait
// 迭代将消耗 GeometryCollection
impl<T: CoordNum> IntoIterator for GeometryCollection<T> {
    type Item = Geometry<T>;
    type IntoIter = IntoIteratorHelper<T>;

    // 注意 into_iter() 正在消耗 self
    fn into_iter(self) -> Self::IntoIter {
        IntoIteratorHelper {
            iter: self.0.into_iter(),
        }
    }
}

// 为辅助结构实现 Iterator trait，以供适配器使用
impl<T: CoordNum> Iterator for IntoIteratorHelper<T> {
    type Item = Geometry<T>;

    // 返回下一个几何图形
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// 非消耗型迭代器的结构辅助
#[derive(Debug)]
pub struct IterHelper<'a, T: CoordNum> {
    iter: ::core::slice::Iter<'a, Geometry<T>>,
}

// 为非消耗型迭代器实现 IntoIterator trait
// 迭代将借用 GeometryCollection
impl<'a, T: CoordNum> IntoIterator for &'a GeometryCollection<T> {
    type Item = &'a Geometry<T>;
    type IntoIter = IterHelper<'a, T>;

    // 注意 into_iter() 正在借用 self
    fn into_iter(self) -> Self::IntoIter {
        IterHelper {
            iter: self.0.iter(),
        }
    }
}

// 为辅助结构实现 Iterator trait，以供适配器使用
impl<'a, T: CoordNum> Iterator for IterHelper<'a, T> {
    type Item = &'a Geometry<T>;

    // 返回下一个几何图形的引用
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// 可变非消耗型迭代器的结构辅助
#[derive(Debug)]
pub struct IterMutHelper<'a, T: CoordNum> {
    iter: ::core::slice::IterMut<'a, Geometry<T>>,
}

// 为可变非消耗型迭代器实现 IntoIterator trait
// 迭代将可变借用 GeometryCollection
impl<'a, T: CoordNum> IntoIterator for &'a mut GeometryCollection<T> {
    type Item = &'a mut Geometry<T>;
    type IntoIter = IterMutHelper<'a, T>;

    // 注意 into_iter() 正在可变借用 self
    fn into_iter(self) -> Self::IntoIter {
        IterMutHelper {
            iter: self.0.iter_mut(),
        }
    }
}

// 为辅助结构实现 Iterator trait，以供适配器使用
impl<'a, T: CoordNum> Iterator for IterMutHelper<'a, T> {
    type Item = &'a mut Geometry<T>;

    // 返回下一个几何图形的可变引用
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, T: CoordNum> GeometryCollection<T> {
    // 返回非消耗型迭代器
    pub fn iter(&'a self) -> IterHelper<'a, T> {
        self.into_iter()
    }

    // 返回可变非消耗型迭代器
    pub fn iter_mut(&'a mut self) -> IterMutHelper<'a, T> {
        self.into_iter()
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for GeometryCollection<T>
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
    /// use geo_types::{GeometryCollection, point};
    ///
    /// let a = GeometryCollection::new_from(vec![point![x: 1.0, y: 2.0].into()]);
    /// let b = GeometryCollection::new_from(vec![point![x: 1.0, y: 2.01].into()]);
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
impl<T> AbsDiffEq for GeometryCollection<T>
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
    /// use geo_types::{GeometryCollection, point};
    ///
    /// let a = GeometryCollection::new_from(vec![point![x: 0.0, y: 0.0].into()]);
    /// let b = GeometryCollection::new_from(vec![point![x: 0.0, y: 0.1].into()]);
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
mod tests {
    use alloc::vec;

    use crate::{GeometryCollection, Point};

    #[test]
    fn from_vec() {
        let gc = GeometryCollection::from(vec![Point::new(1i32, 2)]);
        let p = Point::try_from(gc[0].clone()).unwrap();
        assert_eq!(p.y(), 2);
    }
}
