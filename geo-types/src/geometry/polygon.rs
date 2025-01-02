use crate::{CoordFloat, CoordNum, LineString, Point, Rect, Triangle};
use alloc::vec;
use alloc::vec::Vec;
use num_traits::{Float, Signed};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};

/// 一个有界的二维区域。
///
/// `Polygon`的外边界（外环）由一个[`LineString`]表示。它可能包含零个或多个孔（内环），
/// 这些内环也由`LineString`表示。
///
/// 可以使用[`Polygon::new`]构造函数或[`polygon!`][`crate::polygon!`]宏创建`Polygon`。
///
/// # 语义
///
/// 多边形的边界是外部和内部边界的并集。内部是多边形内部的所有点（不在边界上）。
///
/// `Polygon`结构保证所有外部和内部环都是闭合的，即每个环的第一个和最后一个`Coord`具有相同的值。
///
/// # 有效性
///
/// - 外部和内部环必须是有效的`LinearRing`（参见[`LineString`]）。
///
/// - 边界中的任意两个环不能相交，只能在一个`Point`处作为切线相交。
///   换句话说，环必须是不同的，对于两个环中的每对公共点，
///   必须存在一个包含其中一个点但不包含另一个点的邻域（拓扑开集）。
///
/// - `Polygon`内部的闭包必须等于`Polygon`本身。例如，外部不能包含尖峰。
///
/// - 多边形的内部必须是一个连通的点集。也就是说，内部的任意两个不同点
///   之间必须存在一条位于内部的曲线。
///
/// 有关有效性的正式定义，请参阅OGC-SFA的6.1.11.1节。除了闭合`LineString`保证外，
/// `Polygon`结构目前不强制执行有效性。例如，可以构造一个具有以下特征的`Polygon`：
///
/// - 每个`LineString`环的坐标少于3个
/// - 内部环与其他内部环相交
/// - 内部环延伸超出外部环
///
/// # `LineString`闭合操作
///
/// `Polygon`上的某些API会导致对`LineString`进行闭合操作。操作如下：
///
/// 如果`LineString`的第一个和最后一个`Coord`具有不同的值，
/// 将向`LineString`追加一个新的`Coord`，其值等于第一个`Coord`。
///
/// [`LineString`]: line_string/struct.LineString.html
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Polygon<T: CoordNum = f64> {
    exterior: LineString<T>,
    interiors: Vec<LineString<T>>,
}

impl<T: CoordNum> Polygon<T> {
    /// 使用提供的外部`LineString`环和内部`LineString`环创建一个新的`Polygon`。
    ///
    /// 调用`new`时，外部和内部`LineString`环[将被闭合]。
    ///
    /// [将被闭合]: #linestring-closing-operation
    ///
    /// # 示例
    ///
    /// 创建一个没有内部环的`Polygon`：
    ///
    /// ```
    /// use geo_types::{LineString, Polygon};
    ///
    /// let polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![],
    /// );
    /// ```
    ///
    /// 创建一个带有内部环的`Polygon`：
    ///
    /// ```
    /// use geo_types::{LineString, Polygon};
    ///
    /// let polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![LineString::from(vec![
    ///         (0.1, 0.1),
    ///         (0.9, 0.9),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///     ])],
    /// );
    /// ```
    ///
    /// 如果外部或内部`LineString`的第一个和最后一个`Coord`不再匹配，
    /// 这些`LineString`[将被闭合]：
    ///
    /// ```
    /// use geo_types::{coord, LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(LineString::from(vec![(0., 0.), (1., 1.), (1., 0.)]), vec![]);
    ///
    /// assert_eq!(
    ///     polygon.exterior(),
    ///     &LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.),])
    /// );
    /// ```
    pub fn new(mut exterior: LineString<T>, mut interiors: Vec<LineString<T>>) -> Self {
        exterior.close();
        for interior in &mut interiors {
            interior.close();
        }
        Self {
            exterior,
            interiors,
        }
    }

    /// 消耗`Polygon`，返回外部`LineString`环和内部`LineString`环的向量。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::{LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![LineString::from(vec![
    ///         (0.1, 0.1),
    ///         (0.9, 0.9),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///     ])],
    /// );
    ///
    /// let (exterior, interiors) = polygon.into_inner();
    ///
    /// assert_eq!(
    ///     exterior,
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.),])
    /// );
    ///
    /// assert_eq!(
    ///     interiors,
    ///     vec![LineString::from(vec![
    ///         (0.1, 0.1),
    ///         (0.9, 0.9),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///     ])]
    /// );
    /// ```
    pub fn into_inner(self) -> (LineString<T>, Vec<LineString<T>>) {
        (self.exterior, self.interiors)
    }

    /// 返回外部`LineString`环的引用。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::{LineString, Polygon};
    ///
    /// let exterior = LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]);
    ///
    /// let polygon = Polygon::new(exterior.clone(), vec![]);
    ///
    /// assert_eq!(polygon.exterior(), &exterior);
    /// ```
    pub fn exterior(&self) -> &LineString<T> {
        &self.exterior
    }

    /// 执行提供的闭包`f`，该闭包提供了对外部`LineString`环的可变引用。
    ///
    /// 闭包执行后，外部`LineString`[将被闭合]。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::{coord, LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![],
    /// );
    ///
    /// polygon.exterior_mut(|exterior| {
    ///     exterior.0[1] = coord! { x: 1., y: 2. };
    /// });
    ///
    /// assert_eq!(
    ///     polygon.exterior(),
    ///     &LineString::from(vec![(0., 0.), (1., 2.), (1., 0.), (0., 0.),])
    /// );
    /// ```
    ///
    /// 如果外部`LineString`的第一个和最后一个`Coord`不再匹配，
    /// 该`LineString`[将被闭合]：
    ///
    /// ```
    /// use geo_types::{coord, LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![],
    /// );
    ///
    /// polygon.exterior_mut(|exterior| {
    ///     exterior.0[0] = coord! { x: 0., y: 1. };
    /// });
    ///
    /// assert_eq!(
    ///     polygon.exterior(),
    ///     &LineString::from(vec![(0., 1.), (1., 1.), (1., 0.), (0., 0.), (0., 1.),])
    /// );
    /// ```
    ///
    /// [将被闭合]: #linestring-closing-operation
    pub fn exterior_mut<F>(&mut self, f: F)
    where
        F: FnOnce(&mut LineString<T>),
    {
        f(&mut self.exterior);
        self.exterior.close();
    }

    /// [`exterior_mut`](Polygon::exterior_mut)的可失败替代方案。
    pub fn try_exterior_mut<F, E>(&mut self, f: F) -> Result<(), E>
    where
        F: FnOnce(&mut LineString<T>) -> Result<(), E>,
    {
        f(&mut self.exterior)?;
        self.exterior.close();
        Ok(())
    }

    /// 返回内部`LineString`环的切片。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::{coord, LineString, Polygon};
    ///
    /// let interiors = vec![LineString::from(vec![
    ///     (0.1, 0.1),
    ///     (0.9, 0.9),
    ///     (0.9, 0.1),
    ///     (0.1, 0.1),
    /// ])];
    ///
    /// let polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     interiors.clone(),
    /// );
    ///
    /// assert_eq!(interiors, polygon.interiors());
    /// ```
    pub fn interiors(&self) -> &[LineString<T>] {
        &self.interiors
    }

    /// 执行提供的闭包`f`，该闭包提供了对内部`LineString`环的可变引用。
    ///
    /// 闭包执行后，每个内部`LineString`[将被闭合]。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::{coord, LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![LineString::from(vec![
    ///         (0.1, 0.1),
    ///         (0.9, 0.9),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///     ])],
    /// );
    ///
    /// polygon.interiors_mut(|interiors| {
    ///     interiors[0].0[1] = coord! { x: 0.8, y: 0.8 };
    /// });
    ///
    /// assert_eq!(
    ///     polygon.interiors(),
    ///     &[LineString::from(vec![
    ///         (0.1, 0.1),
    ///         (0.8, 0.8),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///     ])]
    /// );
    /// ```
    ///
    /// 如果任何内部`LineString`的第一个和最后一个`Coord`不再匹配，
    /// 这些`LineString`[将被闭合]：
    ///
    /// ```
    /// use geo_types::{coord, LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![LineString::from(vec![
    ///         (0.1, 0.1),
    ///         (0.9, 0.9),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///     ])],
    /// );
    ///
    /// polygon.interiors_mut(|interiors| {
    ///     interiors[0].0[0] = coord! { x: 0.1, y: 0.2 };
    /// });
    ///
    /// assert_eq!(
    ///     polygon.interiors(),
    ///     &[LineString::from(vec![
    ///         (0.1, 0.2),
    ///         (0.9, 0.9),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///         (0.1, 0.2),
    ///     ])]
    /// );
    /// ```
    ///
    /// [将被闭合]: #linestring-closing-operation
    pub fn interiors_mut<F>(&mut self, f: F)
    where
        F: FnOnce(&mut [LineString<T>]),
    {
        f(&mut self.interiors);
        for interior in &mut self.interiors {
            interior.close();
        }
    }

    /// Fallible alternative to [`interiors_mut`](Self::interiors_mut).
    pub fn try_interiors_mut<F, E>(&mut self, f: F) -> Result<(), E>
    where
        F: FnOnce(&mut [LineString<T>]) -> Result<(), E>,
    {
        f(&mut self.interiors)?;
        for interior in &mut self.interiors {
            interior.close();
        }
        Ok(())
    }

    /// Add an interior ring to the `Polygon`.
    ///
    /// The new `LineString` interior ring [will be closed]:
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{coord, LineString, Polygon};
    ///
    /// let mut polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![],
    /// );
    ///
    /// assert_eq!(polygon.interiors().len(), 0);
    ///
    /// polygon.interiors_push(vec![(0.1, 0.1), (0.9, 0.9), (0.9, 0.1)]);
    ///
    /// assert_eq!(
    ///     polygon.interiors(),
    ///     &[LineString::from(vec![
    ///         (0.1, 0.1),
    ///         (0.9, 0.9),
    ///         (0.9, 0.1),
    ///         (0.1, 0.1),
    ///     ])]
    /// );
    /// ```
    ///
    /// [will be closed]: #linestring-closing-operation
    pub fn interiors_push(&mut self, new_interior: impl Into<LineString<T>>) {
        let mut new_interior = new_interior.into();
        new_interior.close();
        self.interiors.push(new_interior);
    }

    /// Wrap-around previous-vertex
    fn previous_vertex(&self, current_vertex: usize) -> usize
    where
        T: Float,
    {
        (current_vertex + (self.exterior.0.len() - 1) - 1) % (self.exterior.0.len() - 1)
    }

    /// Count the total number of rings (interior and exterior) in the polygon
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{coord, LineString, Polygon};
    ///
    /// let polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![],
    /// );
    ///
    /// assert_eq!(polygon.num_rings(), 1);
    ///
    /// let polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![LineString::from(vec![(0.1, 0.1), (0.9, 0.9), (0.9, 0.1)])],
    /// );
    ///
    /// assert_eq!(polygon.num_rings(), 2);
    /// ```
    pub fn num_rings(&self) -> usize {
        self.num_interior_rings() + 1
    }

    /// Count the number of interior rings in the polygon
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{coord, LineString, Polygon};
    ///
    /// let polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![],
    /// );
    ///
    /// assert_eq!(polygon.num_interior_rings(), 0);
    ///
    /// let polygon = Polygon::new(
    ///     LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]),
    ///     vec![LineString::from(vec![(0.1, 0.1), (0.9, 0.9), (0.9, 0.1)])],
    /// );
    ///
    /// assert_eq!(polygon.num_interior_rings(), 1);
    /// ```
    pub fn num_interior_rings(&self) -> usize {
        self.interiors.len()
    }
}

// used to check the sign of a vec of floats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ListSign {
    Empty,
    Positive,
    Negative,
    Mixed,
}

impl<T: CoordFloat + Signed> Polygon<T> {
    /// Determine whether a Polygon is convex
    // For each consecutive pair of edges of the polygon (each triplet of points),
    // compute the z-component of the cross product of the vectors defined by the
    // edges pointing towards the points in increasing order.
    // Take the cross product of these vectors
    // The polygon is convex if the z-components of the cross products are either
    // all positive or all negative. Otherwise, the polygon is non-convex.
    // see: http://stackoverflow.com/a/1881201/416626
    #[deprecated(
        since = "0.6.1",
        note = "Please use `geo::is_convex` on `poly.exterior()` instead"
    )]
    pub fn is_convex(&self) -> bool {
        let convex = self
            .exterior
            .0
            .iter()
            .enumerate()
            .map(|(idx, _)| {
                let prev_1 = self.previous_vertex(idx);
                let prev_2 = self.previous_vertex(prev_1);
                Point::from(self.exterior[prev_2]).cross_prod(
                    Point::from(self.exterior[prev_1]),
                    Point::from(self.exterior[idx]),
                )
            })
            // accumulate and check cross-product result signs in a single pass
            // positive implies ccw convexity, negative implies cw convexity
            // anything else implies non-convexity
            .fold(ListSign::Empty, |acc, n| match (acc, n.is_positive()) {
                (ListSign::Empty, true) | (ListSign::Positive, true) => ListSign::Positive,
                (ListSign::Empty, false) | (ListSign::Negative, false) => ListSign::Negative,
                _ => ListSign::Mixed,
            });
        convex != ListSign::Mixed
    }
}

impl<T: CoordNum> From<Rect<T>> for Polygon<T> {
    fn from(r: Rect<T>) -> Self {
        Polygon::new(
            vec![
                (r.min().x, r.min().y),
                (r.max().x, r.min().y),
                (r.max().x, r.max().y),
                (r.min().x, r.max().y),
                (r.min().x, r.min().y),
            ]
            .into(),
            Vec::new(),
        )
    }
}

impl<T: CoordNum> From<Triangle<T>> for Polygon<T> {
    fn from(t: Triangle<T>) -> Self {
        Polygon::new(vec![t.0, t.1, t.2, t.0].into(), Vec::new())
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for Polygon<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    /// Equality assertion within a relative limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Polygon, polygon};
    ///
    /// let a: Polygon<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7., y: 9.), (x: 0., y: 0.)];
    /// let b: Polygon<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7.01, y: 9.), (x: 0., y: 0.)];
    ///
    /// approx::assert_relative_eq!(a, b, max_relative=0.1);
    /// approx::assert_relative_ne!(a, b, max_relative=0.001);
    /// ```
    ///
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if !self
            .exterior
            .relative_eq(&other.exterior, epsilon, max_relative)
        {
            return false;
        }

        if self.interiors.len() != other.interiors.len() {
            return false;
        }
        let mut zipper = self.interiors.iter().zip(other.interiors.iter());
        zipper.all(|(lhs, rhs)| lhs.relative_eq(rhs, epsilon, max_relative))
    }
}

#[cfg(any(feature = "approx", test))]
impl<T: AbsDiffEq<Epsilon = T> + CoordNum> AbsDiffEq for Polygon<T> {
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    /// Equality assertion with an absolute limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Polygon, polygon};
    ///
    /// let a: Polygon<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7., y: 9.), (x: 0., y: 0.)];
    /// let b: Polygon<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7.01, y: 9.), (x: 0., y: 0.)];
    ///
    /// approx::assert_abs_diff_eq!(a, b, epsilon=0.1);
    /// approx::assert_abs_diff_ne!(a, b, epsilon=0.001);
    /// ```
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if !self.exterior.abs_diff_eq(&other.exterior, epsilon) {
            return false;
        }

        if self.interiors.len() != other.interiors.len() {
            return false;
        }
        let mut zipper = self.interiors.iter().zip(other.interiors.iter());
        zipper.all(|(lhs, rhs)| lhs.abs_diff_eq(rhs, epsilon))
    }
}

#[cfg(any(
    feature = "rstar_0_8",
    feature = "rstar_0_9",
    feature = "rstar_0_10",
    feature = "rstar_0_11",
    feature = "rstar_0_12"
))]
macro_rules! impl_rstar_polygon {
    ($rstar:ident) => {
        impl<T> $rstar::RTreeObject for Polygon<T>
        where
            T: ::num_traits::Float + ::$rstar::RTreeNum,
        {
            type Envelope = ::$rstar::AABB<Point<T>>;

            fn envelope(&self) -> Self::Envelope {
                self.exterior.envelope()
            }
        }
    };
}

#[cfg(feature = "rstar_0_8")]
impl_rstar_polygon!(rstar_0_8);

#[cfg(feature = "rstar_0_9")]
impl_rstar_polygon!(rstar_0_9);

#[cfg(feature = "rstar_0_10")]
impl_rstar_polygon!(rstar_0_10);

#[cfg(feature = "rstar_0_11")]
impl_rstar_polygon!(rstar_0_11);

#[cfg(feature = "rstar_0_12")]
impl_rstar_polygon!(rstar_0_12);
