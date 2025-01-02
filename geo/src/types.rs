use crate::{GeoFloat, Point};

/// 尝试在对象上找到离某点最近点的结果。
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Closest<F: GeoFloat> {
    /// 实际上与对象相交的点。
    Intersection(Point<F>),
    /// 在该对象上只有一个地方最接近该点。
    SinglePoint(Point<F>),
    /// 存在两个或更多（可能是无限或未定义）的可能点。
    Indeterminate,
}

impl<F: GeoFloat> Closest<F> {
    /// 相对于 `p` 比较两个 `Closest` 并返回较优者的副本。
    pub fn best_of_two(&self, other: &Self, p: Point<F>) -> Self {
        use crate::{Distance, Euclidean};

        let left = match *self {
            Closest::Indeterminate => return *other,
            Closest::Intersection(_) => return *self,
            Closest::SinglePoint(l) => l,
        };
        let right = match *other {
            Closest::Indeterminate => return *self,
            Closest::Intersection(_) => return *other,
            Closest::SinglePoint(r) => r,
        };

        if Euclidean::distance(left, p) <= Euclidean::distance(right, p) {
            *self
        } else {
            *other
        }
    }
}

/// 实现一个常见的模式，即 Geometry 枚举简单地将其特性实现委托给其内部类型。
///
/// ```
/// # use geo::{GeoNum, Coord, Point, Line, LineString, Polygon, MultiPoint, MultiLineString, MultiPolygon, GeometryCollection, Rect, Triangle, Geometry};
///
/// trait Foo<T: GeoNum> {
///     fn foo_1(&self, coord: Coord<T>) -> bool;
///     fn foo_2(&self) -> i32;
/// }
///
/// // 假设我们为所有的内部类型实现了如下：
/// impl<T: GeoNum> Foo<T> for Point<T> {
///     fn foo_1(&self, coord: Coord<T>) -> bool { true }
///     fn foo_2(&self) -> i32 { 1 }
/// }
/// impl<T: GeoNum> Foo<T> for Line<T> {
///     fn foo_1(&self, coord: Coord<T>) -> bool { false }
///     fn foo_2(&self) -> i32 { 2 }
/// }
/// impl<T: GeoNum> Foo<T> for LineString<T> {
///     fn foo_1(&self, coord: Coord<T>) -> bool { true }
///     fn foo_2(&self) -> i32 { 3 }
/// }
/// impl<T: GeoNum> Foo<T> for Polygon<T> {
///     fn foo_1(&self, coord: Coord<T>) -> bool { false }
///     fn foo_2(&self) -> i32 { 4 }
/// }
/// impl<T: GeoNum> Foo<T> for MultiPoint<T> {
///     fn foo_1(&self, coord: Coord<T>) -> bool { true }
///     fn foo_2(&self) -> i32 { 5 }
/// }
/// impl<T: GeoNum> Foo<T> for MultiLineString<T> {
///     fn foo_1(&self, coord: Coord<T>) -> bool { false }
///     fn foo_2(&self) -> i32 { 6 }
/// }
/// impl<T: GeoNum> Foo<T> for MultiPolygon<T> {
///     fn foo_1(&self, coord: Coord<T>) -> bool { true }
///     fn foo_2(&self) -> i32 { 7 }
/// }
/// impl<T: GeoNum> Foo<T> for GeometryCollection<T> {
///     fn foo_1(&self, coord: Coord<T>) -> bool { false }
///     fn foo_2(&self) -> i32 { 8 }
/// }
/// impl<T: GeoNum> Foo<T> for Rect<T> {
///     fn foo_1(&self, coord: Coord<T>) -> bool { true }
///     fn foo_2(&self) -> i32 { 9 }
/// }
/// impl<T: GeoNum> Foo<T> for Triangle<T> {
///     fn foo_1(&self, coord: Coord<T>) -> bool { true }
///     fn foo_2(&self) -> i32 { 10 }
/// }
///
/// // 如果我们想要为 Geometry 的实现简单地委托到其内部情形...
/// impl<T: GeoNum> Foo<T> for Geometry<T> {
///     // 代替写出这些简单的枚举委托...
///     fn foo_1(&self, coord: Coord<T>) -> bool {
///         match self {
///            Geometry::Point(g) => g.foo_1(coord),
///            Geometry::LineString(g) => g.foo_1(coord),
///            _ => unimplemented!("...etc for other cases")
///         }
///     }
///    
///     fn foo_2(&self) -> i32 {
///         match self {
///            Geometry::Point(g) => g.foo_2(),
///            Geometry::LineString(g) => g.foo_2(),
///            _ => unimplemented!("...etc for other cases")
///         }
///     }
///
///     // 我们可以等效地写作：
/// impl<T: GeoNum> Foo<T> for Geometry<T> {
///     geo::geometry_delegate_impl! {
///         fn foo_1(&self, coord: Coord<T>) -> bool;
///         fn foo_2(&self) -> i32;
///     }
/// }
#[macro_export]
macro_rules! geometry_delegate_impl {
    ($($a:tt)*) => { $crate::__geometry_delegate_impl_helper!{ Geometry, $($a)* } }
}

#[doc(hidden)]
#[macro_export]
macro_rules! geometry_cow_delegate_impl {
    ($($a:tt)*) => { $crate::__geometry_delegate_impl_helper!{ GeometryCow, $($a)* } }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __geometry_delegate_impl_helper {
    (
        $enum:ident,
        $(
            $(#[$outer:meta])*
            fn $func_name: ident(&$($self_life:lifetime)?self $(, $arg_name: ident: $arg_type: ty)*) -> $return: ty;
         )+
    ) => {
            $(
                $(#[$outer])*
                fn $func_name(&$($self_life)? self, $($arg_name: $arg_type),*) -> $return {
                    match self {
                        $enum::Point(g) => g.$func_name($($arg_name),*).into(),
                        $enum::Line(g) =>  g.$func_name($($arg_name),*).into(),
                        $enum::LineString(g) => g.$func_name($($arg_name),*).into(),
                        $enum::Polygon(g) => g.$func_name($($arg_name),*).into(),
                        $enum::MultiPoint(g) => g.$func_name($($arg_name),*).into(),
                        $enum::MultiLineString(g) => g.$func_name($($arg_name),*).into(),
                        $enum::MultiPolygon(g) => g.$func_name($($arg_name),*).into(),
                        $enum::GeometryCollection(g) => g.$func_name($($arg_name),*).into(),
                        $enum::Rect(g) => g.$func_name($($arg_name),*).into(),
                        $enum::Triangle(g) => g.$func_name($($arg_name),*).into(),
                    }
                }
            )+
        };
}
