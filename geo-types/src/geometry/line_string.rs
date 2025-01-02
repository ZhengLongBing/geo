#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};

use crate::{Coord, CoordNum, Line, Point, Triangle};
use alloc::vec;
use alloc::vec::Vec;
use core::iter::FromIterator;
use core::ops::{Index, IndexMut};

/// [`Coord`]的有序集合，表示位置之间的路径。
/// 要有效，`LineString`必须为空，或者有两个或更多的坐标。
///
/// # 语义
///
/// 1. 如果[`LineString`]为空，**或者**第一个和最后一个坐标相同，则它是_闭合的_。
/// 2. [`LineString`]的_边界_是：
///     - 如果它是_闭合的_（见**1**），则为**空**，**或者**
///     - 包含**起始**和**结束**坐标。
/// 3. _内部_是沿[`LineString`]的所有坐标的（无限）集合，_不包括_边界。
/// 4. 如果[`LineString`]除了**可选地**在第一个和最后一个坐标处（在这种情况下它也是_闭合的_，见**1**）之外不相交，则它是_简单的_。
/// 5. _简单的_**且**_闭合的_[`LineString`]是OGC-SFA中定义的`LinearRing`（但在这个crate中没有定义为单独的类型）。
///
/// # 有效性
///
/// 如果[`LineString`]为空或包含2个或更多坐标，则它是有效的。
///
/// 此外，闭合的[`LineString`]**不能**自相交。注意，它的
/// 有效性**不**被强制执行，对无效的`LineString`进行操作和
/// 谓词是**未定义的**。
///
/// # 示例
/// ## 创建
///
/// 通过直接调用来创建[`LineString`]：
///
/// ```
/// use geo_types::{coord, LineString};
///
/// let line_string = LineString::new(vec![
///     coord! { x: 0., y: 0. },
///     coord! { x: 10., y: 0. },
/// ]);
/// ```
///
/// 使用[`line_string!`][`crate::line_string!`]宏创建[`LineString`]：
///
/// ```
/// use geo_types::line_string;
///
/// let line_string = line_string![
///     (x: 0., y: 0.),
///     (x: 10., y: 0.),
/// ];
/// ```
///
/// 通过转换坐标类对象的[`Vec`]：
///
/// ```
/// use geo_types::LineString;
///
/// let line_string: LineString<f32> = vec![(0., 0.), (10., 0.)].into();
/// ```
///
/// ```
/// use geo_types::LineString;
///
/// let line_string: LineString = vec![[0., 0.], [10., 0.]].into();
/// ```
//
/// 或者通过从[`Coord`]迭代器`collect`：
///
/// ```
/// use geo_types::{coord, LineString};
///
/// let mut coords_iter =
///     vec![coord! { x: 0., y: 0. }, coord! { x: 10., y: 0. }].into_iter();
///
/// let line_string: LineString<f32> = coords_iter.collect();
/// ```
///
/// ## 迭代
/// [`LineString`]提供五个迭代器：[`coords`](LineString::coords)、[`coords_mut`](LineString::coords_mut)、[`points`](LineString::points)、[`lines`](LineString::lines)和[`triangles`](LineString::triangles)：
///
/// ```
/// use geo_types::{coord, LineString};
///
/// let line_string = LineString::new(vec![
///     coord! { x: 0., y: 0. },
///     coord! { x: 10., y: 0. },
/// ]);
///
/// line_string.coords().for_each(|coord| println!("{:?}", coord));
///
/// for point in line_string.points() {
///     println!("Point x = {}, y = {}", point.x(), point.y());
/// }
/// ```
///
/// 注意，它的[`IntoIterator`]实现在循环时会产生[`Coord`]：
///
/// ```
/// use geo_types::{coord, LineString};
///
/// let line_string = LineString::new(vec![
///     coord! { x: 0., y: 0. },
///     coord! { x: 10., y: 0. },
/// ]);
///
/// for coord in &line_string {
///     println!("Coordinate x = {}, y = {}", coord.x, coord.y);
/// }
///
/// for coord in line_string {
///     println!("Coordinate x = {}, y = {}", coord.x, coord.y);
/// }
///
/// ```
/// ## 分解
///
/// 你可以将[`LineString`]分解为[`Coord`]或[`Point`]的[`Vec`]：
/// ```
/// use geo_types::{coord, LineString, Point};
///
/// let line_string = LineString::new(vec![
///     coord! { x: 0., y: 0. },
///     coord! { x: 10., y: 0. },
/// ]);
///
/// let coordinate_vec = line_string.clone().into_inner();
/// let point_vec = line_string.clone().into_points();
///
/// ```

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LineString<T: CoordNum = f64>(pub Vec<Coord<T>>);

/// 由`points`方法返回的[`Point`]迭代器
#[derive(Debug)]
pub struct PointsIter<'a, T: CoordNum + 'a>(::core::slice::Iter<'a, Coord<T>>);

impl<T: CoordNum> Iterator for PointsIter<'_, T> {
    type Item = Point<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| Point::from(*c))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T: CoordNum> ExactSizeIterator for PointsIter<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: CoordNum> DoubleEndedIterator for PointsIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|c| Point::from(*c))
    }
}

/// 由[`LineString`]的`into_iter`方法使用的[`Coord`]迭代器
#[derive(Debug)]
pub struct CoordinatesIter<'a, T: CoordNum + 'a>(::core::slice::Iter<'a, Coord<T>>);

impl<'a, T: CoordNum> Iterator for CoordinatesIter<'a, T> {
    type Item = &'a Coord<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T: CoordNum> ExactSizeIterator for CoordinatesIter<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: CoordNum> DoubleEndedIterator for CoordinatesIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T: CoordNum> LineString<T> {
    /// 从原始内容值实例化Self
    pub fn new(value: Vec<Coord<T>>) -> Self {
        Self(value)
    }

    /// 返回一个迭代器，产生[`LineString`]的坐标作为[`Point`]
    #[deprecated(note = "使用points()代替")]
    pub fn points_iter(&self) -> PointsIter<T> {
        PointsIter(self.0.iter())
    }

    /// 返回一个迭代器，产生[`LineString`]的坐标作为[`Point`]
    pub fn points(&self) -> PointsIter<T> {
        PointsIter(self.0.iter())
    }

    /// 返回一个迭代器，产生[`LineString`]的成员作为[`Coord`]
    pub fn coords(&self) -> impl DoubleEndedIterator<Item = &Coord<T>> {
        self.0.iter()
    }

    /// 返回一个迭代器，产生[`LineString`]的坐标作为可变的[`Coord`]
    pub fn coords_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Coord<T>> {
        self.0.iter_mut()
    }

    /// 将[`LineString`]的坐标作为[`Point`]的[`Vec`]返回
    pub fn into_points(self) -> Vec<Point<T>> {
        self.0.into_iter().map(Point::from).collect()
    }

    /// 将[`LineString`]的坐标作为[`Coord`]的[`Vec`]返回
    pub fn into_inner(self) -> Vec<Coord<T>> {
        self.0
    }

    /// 返回一个迭代器，为[`LineString`]中的每个线段产生一个[Line]。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::{coord, Line, LineString};
    ///
    /// let mut coords = vec![(0., 0.), (5., 0.), (7., 9.)];
    /// let line_string: LineString<f32> = coords.into_iter().collect();
    ///
    /// let mut lines = line_string.lines();
    /// assert_eq!(
    ///     Some(Line::new(
    ///         coord! { x: 0., y: 0. },
    ///         coord! { x: 5., y: 0. }
    ///     )),
    ///     lines.next()
    /// );
    /// assert_eq!(
    ///     Some(Line::new(
    ///         coord! { x: 5., y: 0. },
    ///         coord! { x: 7., y: 9. }
    ///     )),
    ///     lines.next()
    /// );
    /// assert!(lines.next().is_none());
    /// ```
    pub fn lines(&'_ self) -> impl ExactSizeIterator<Item = Line<T>> + '_ {
        self.0.windows(2).map(|w| {
            // slice::windows(N)保证产生一个恰好有N个元素的切片
            unsafe { Line::new(*w.get_unchecked(0), *w.get_unchecked(1)) }
        })
    }

    /// 一个迭代器，产生[`LineString`]的坐标作为[Triangle]
    pub fn triangles(&'_ self) -> impl ExactSizeIterator<Item = Triangle<T>> + '_ {
        self.0.windows(3).map(|w| {
            // slice::windows(N)保证产生一个恰好有N个元素的切片
            unsafe {
                Triangle::new(
                    *w.get_unchecked(0),
                    *w.get_unchecked(1),
                    *w.get_unchecked(2),
                )
            }
        })
    }

    /// 闭合[`LineString`]。具体来说，如果[`LineString`]至少有一个[`Coord`]，
    /// 且第一个[`Coord`]的值**不等于**最后一个[`Coord`]的值，
    /// 则在末尾添加一个新的[`Coord`]，其值等于第一个[`Coord`]。
    pub fn close(&mut self) {
        if !self.is_closed() {
            // 根据定义，我们将空的LineString视为闭合。
            debug_assert!(!self.0.is_empty());
            self.0.push(self.0[0]);
        }
    }

    /// 返回[`LineString`]中的坐标数量。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::LineString;
    ///
    /// let mut coords = vec![(0., 0.), (5., 0.), (7., 9.)];
    /// let line_string: LineString<f32> = coords.into_iter().collect();
    ///
    /// # #[allow(deprecated)]
    /// # {
    /// assert_eq!(3, line_string.num_coords());
    /// # }
    /// ```
    #[deprecated(note = "使用geo::CoordsIter::coords_count代替")]
    pub fn num_coords(&self) -> usize {
        self.0.len()
    }

    /// 检查linestring是否闭合；即它是
    /// 空的或者第一个和最后一个点相同。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::LineString;
    ///
    /// let mut coords = vec![(0., 0.), (5., 0.), (0., 0.)];
    /// let line_string: LineString<f32> = coords.into_iter().collect();
    /// assert!(line_string.is_closed());
    /// ```
    ///
    /// 注意，我们与一些库（[JTS](https://locationtech.github.io/jts/javadoc/org/locationtech/jts/geom/LinearRing.html)等）有所不同，
    /// 这些库有一个单独的`LinearRing`类型，与[`LineString`]分开。这些库将空的`LinearRing`定义为**闭合**，
    /// 而将空的`LineString`视为**开放**。由于我们没有单独的`LinearRing`类型，
    /// 而是在其位置使用[`LineString`]，我们在所有地方采用JTS `LinearRing` `is_closed`
    /// 行为：也就是说，**我们将空的[`LineString`]视为闭合**。
    ///
    /// 这在用作[`Polygon.exterior`](crate::Polygon::exterior)和其他地方时是预期的；
    /// 注意，我们与一些库（如[JTS](https://locationtech.github.io/jts/javadoc/org/locationtech/jts/geom/LinearRing.html)等）有所不同，
    /// 这些库有一个单独的`LinearRing`类型，与[`LineString`]分开。这些库将空的`LinearRing`定义为**闭合**，
    /// 而将空的`LineString`视为**开放**。由于我们没有单独的`LinearRing`类型，
    /// 而是在其位置使用[`LineString`]，我们在所有地方采用JTS `LinearRing` `is_closed`
    /// 行为：也就是说，**我们将空的[`LineString`]视为闭合**。
    ///
    /// 这在用作[`Polygon.exterior`](crate::Polygon::exterior)和其他地方时是预期的；并且
    /// 似乎没有理由为在非`LinearRing`上下文中使用的[`LineString`]保持单独的行为。
    pub fn is_closed(&self) -> bool {
        self.0.first() == self.0.last()
    }
}

/// 将[`Point`]类对象的[`Vec`]转换为[`LineString`]。
impl<T: CoordNum, IC: Into<Coord<T>>> From<Vec<IC>> for LineString<T> {
    fn from(v: Vec<IC>) -> Self {
        Self(v.into_iter().map(|c| c.into()).collect())
    }
}

impl<T: CoordNum> From<Line<T>> for LineString<T> {
    fn from(line: Line<T>) -> Self {
        LineString::from(&line)
    }
}

impl<T: CoordNum> From<&Line<T>> for LineString<T> {
    fn from(line: &Line<T>) -> Self {
        Self(vec![line.start, line.end])
    }
}

/// 将[`Point`]类对象的迭代器转换为[`LineString`]。
impl<T: CoordNum, IC: Into<Coord<T>>> FromIterator<IC> for LineString<T> {
    fn from_iter<I: IntoIterator<Item = IC>>(iter: I) -> Self {
        Self(iter.into_iter().map(|c| c.into()).collect())
    }
}

/// 遍历此[`LineString`]中的所有[`Coord`]。
impl<T: CoordNum> IntoIterator for LineString<T> {
    type Item = Coord<T>;
    type IntoIter = ::alloc::vec::IntoIter<Coord<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a LineString<T> {
    type Item = &'a Coord<T>;
    type IntoIter = CoordinatesIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        CoordinatesIter(self.0.iter())
    }
}

/// 可变地遍历此[`LineString`]中的所有[`Coord`]
impl<'a, T: CoordNum> IntoIterator for &'a mut LineString<T> {
    type Item = &'a mut Coord<T>;
    type IntoIter = ::core::slice::IterMut<'a, Coord<T>>;

    fn into_iter(self) -> ::core::slice::IterMut<'a, Coord<T>> {
        self.0.iter_mut()
    }
}

impl<T: CoordNum> Index<usize> for LineString<T> {
    type Output = Coord<T>;

    fn index(&self, index: usize) -> &Coord<T> {
        self.0.index(index)
    }
}

impl<T: CoordNum> IndexMut<usize> for LineString<T> {
    fn index_mut(&mut self, index: usize) -> &mut Coord<T> {
        self.0.index_mut(index)
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for LineString<T>
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
    /// use geo_types::LineString;
    ///
    /// let mut coords_a = vec![(0., 0.), (5., 0.), (7., 9.)];
    /// let a: LineString<f32> = coords_a.into_iter().collect();
    ///
    /// let mut coords_b = vec![(0., 0.), (5., 0.), (7.001, 9.)];
    /// let b: LineString<f32> = coords_b.into_iter().collect();
    ///
    /// approx::assert_relative_eq!(a, b, max_relative=0.1)
    /// ```
    ///
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        let points_zipper = self.points().zip(other.points());
        for (lhs, rhs) in points_zipper {
            if lhs.relative_ne(&rhs, epsilon, max_relative) {
                return false;
            }
        }

        true
    }
}

#[cfg(any(feature = "approx", test))]
impl<T: AbsDiffEq<Epsilon = T> + CoordNum> AbsDiffEq for LineString<T> {
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    /// 具有绝对限制的相等性断言。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::LineString;
    ///
    /// let mut coords_a = vec![(0., 0.), (5., 0.), (7., 9.)];
    /// let a: LineString<f32> = coords_a.into_iter().collect();
    ///
    /// let mut coords_b = vec![(0., 0.), (5., 0.), (7.001, 9.)];
    /// let b: LineString<f32> = coords_b.into_iter().collect();
    ///
    /// approx::assert_relative_eq!(a, b, epsilon=0.1)
    /// ```
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        let mut points_zipper = self.points().zip(other.points());
        points_zipper.all(|(lhs, rhs)| lhs.abs_diff_eq(&rhs, epsilon))
    }
}

#[cfg(any(
    feature = "rstar_0_8",
    feature = "rstar_0_9",
    feature = "rstar_0_10",
    feature = "rstar_0_11",
    feature = "rstar_0_12"
))]
macro_rules! impl_rstar_line_string {
    ($rstar:ident) => {
        impl<T> ::$rstar::RTreeObject for LineString<T>
        where
            T: ::num_traits::Float + ::$rstar::RTreeNum,
        {
            type Envelope = ::$rstar::AABB<Point<T>>;

            fn envelope(&self) -> Self::Envelope {
                use num_traits::Bounded;
                let bounding_rect = crate::private_utils::line_string_bounding_rect(self);
                match bounding_rect {
                    None => ::$rstar::AABB::from_corners(
                        Point::new(Bounded::min_value(), Bounded::min_value()),
                        Point::new(Bounded::max_value(), Bounded::max_value()),
                    ),
                    Some(b) => ::$rstar::AABB::from_corners(
                        Point::new(b.min().x, b.min().y),
                        Point::new(b.max().x, b.max().y),
                    ),
                }
            }
        }

        impl<T> ::$rstar::PointDistance for LineString<T>
        where
            T: ::num_traits::Float + ::$rstar::RTreeNum,
        {
            fn distance_2(&self, point: &Point<T>) -> T {
                let d = crate::private_utils::point_line_string_euclidean_distance(*point, self);
                if d == T::zero() {
                    d
                } else {
                    d.powi(2)
                }
            }
        }
    };
}

#[cfg(feature = "rstar_0_8")]
impl_rstar_line_string!(rstar_0_8);

#[cfg(feature = "rstar_0_9")]
impl_rstar_line_string!(rstar_0_9);

#[cfg(feature = "rstar_0_10")]
impl_rstar_line_string!(rstar_0_10);

#[cfg(feature = "rstar_0_11")]
impl_rstar_line_string!(rstar_0_11);

#[cfg(feature = "rstar_0_12")]
impl_rstar_line_string!(rstar_0_12);

#[cfg(test)]
mod test {
    use super::*;
    use crate::coord;
    use approx::AbsDiffEq;

    #[test]
    fn test_exact_size() {
        // 参见 https://github.com/georust/geo/issues/762
        let first = coord! { x: 0., y: 0. };
        let ls = LineString::new(vec![first, coord! { x: 10., y: 0. }]);

        // 引用以强制使用 `impl IntoIterator for &LineString` 实现，得到 `CoordinatesIter`
        for c in (&ls).into_iter().rev().skip(1).rev() {
            assert_eq!(&first, c);
        }
        for p in ls.points().rev().skip(1).rev() {
            assert_eq!(Point::from(first), p);
        }
    }

    #[test]
    fn test_abs_diff_eq() {
        let delta = 1e-6;

        let coords = vec![(0., 0.), (5., 0.), (10., 10.)];
        let ls: LineString<f32> = coords.into_iter().collect();

        let coords_x = vec![(0., 0.), (5. + delta, 0.), (10., 10.)];
        let ls_x: LineString<f32> = coords_x.into_iter().collect();
        assert!(ls.abs_diff_eq(&ls_x, 1e-2));
        assert!(ls.abs_diff_ne(&ls_x, 1e-12));

        let coords_y = vec![(0., 0.), (5., 0. + delta), (10., 10.)];
        let ls_y: LineString<f32> = coords_y.into_iter().collect();
        assert!(ls.abs_diff_eq(&ls_y, 1e-2));
        assert!(ls.abs_diff_ne(&ls_y, 1e-12));

        // 长度不足，但其他方面相等。
        let coords_x = vec![(0., 0.), (5., 0.)];
        let ls_under: LineString<f32> = coords_x.into_iter().collect();
        assert!(ls.abs_diff_ne(&ls_under, 1.));

        // 长度过长，但其他方面相等。
        let coords_x = vec![(0., 0.), (5., 0.), (10., 10.), (10., 100.)];
        let ls_oversized: LineString<f32> = coords_x.into_iter().collect();
        assert!(ls.abs_diff_ne(&ls_oversized, 1.));
    }

    #[test]
    fn test_relative_eq() {
        let delta = 1e-6;

        let coords = vec![(0., 0.), (5., 0.), (10., 10.)];
        let ls: LineString<f32> = coords.into_iter().collect();

        let coords_x = vec![(0., 0.), (5. + delta, 0.), (10., 10.)];
        let ls_x: LineString<f32> = coords_x.into_iter().collect();
        assert!(ls.relative_eq(&ls_x, 1e-2, 1e-2));
        assert!(ls.relative_ne(&ls_x, 1e-12, 1e-12));

        let coords_y = vec![(0., 0.), (5., 0. + delta), (10., 10.)];
        let ls_y: LineString<f32> = coords_y.into_iter().collect();
        assert!(ls.relative_eq(&ls_y, 1e-2, 1e-2));
        assert!(ls.relative_ne(&ls_y, 1e-12, 1e-12));

        // 长度不足，但其他方面相等。
        let coords_x = vec![(0., 0.), (5., 0.)];
        let ls_under: LineString<f32> = coords_x.into_iter().collect();
        assert!(ls.relative_ne(&ls_under, 1., 1.));

        // 长度过长，但其他方面相等。
        let coords_x = vec![(0., 0.), (5., 0.), (10., 10.), (10., 100.)];
        let ls_oversized: LineString<f32> = coords_x.into_iter().collect();
        assert!(ls.relative_ne(&ls_oversized, 1., 1.));
    }

    #[test]
    fn should_be_built_from_line() {
        let start = coord! { x: 0, y: 0 };
        let end = coord! { x: 10, y: 10 };
        let line = Line::new(start, end);
        let expected = LineString::new(vec![start, end]);

        assert_eq!(expected, LineString::from(line));

        let start = coord! { x: 10., y: 0.5 };
        let end = coord! { x: 10000., y: 10.4 };
        let line = Line::new(start, end);
        let expected = LineString::new(vec![start, end]);

        assert_eq!(expected, LineString::from(line));
    }
}
