pub(crate) mod coord;
pub(crate) mod geometry_collection;
pub(crate) mod line;
pub(crate) mod line_string;
pub(crate) mod multi_line_string;
pub(crate) mod multi_point;
pub(crate) mod multi_polygon;
pub(crate) mod point;
pub(crate) mod polygon;
pub(crate) mod rect;
pub(crate) mod triangle;

// 重新导出所有几何变体：
#[allow(deprecated)]
pub use coord::{Coord, Coordinate};
pub use geometry_collection::GeometryCollection;
pub use line::Line;
pub use line_string::LineString;
pub use multi_line_string::MultiLineString;
pub use multi_point::MultiPoint;
pub use multi_polygon::MultiPolygon;
pub use point::Point;
pub use polygon::Polygon;
pub use rect::Rect;
pub use triangle::Triangle;

use crate::{CoordNum, Error};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};
use core::any::type_name;
use core::convert::TryFrom;

/// 表示任何可能的几何类型的枚举。
///
/// 所有几何变体（[`Point`]、[`LineString`]等）都可以使用[`Into::into`]转换为`Geometry`。
/// 相反，[`TryFrom::try_from`]可以用于将[`Geometry`]转换回其特定的枚举成员。
///
/// # 示例
///
/// ```
/// use std::convert::TryFrom;
/// use geo_types::{Point, point, Geometry, GeometryCollection};
/// let p = point!(x: 1.0, y: 1.0);
/// let pe: Geometry = p.into();
/// let pn = Point::try_from(pe).unwrap();
/// ```
///
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Geometry<T: CoordNum = f64> {
    Point(Point<T>),
    Line(Line<T>),
    LineString(LineString<T>),
    Polygon(Polygon<T>),
    MultiPoint(MultiPoint<T>),
    MultiLineString(MultiLineString<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>),
    Rect(Rect<T>),
    Triangle(Triangle<T>),
}

// 为各种几何类型实现From trait
impl<T: CoordNum> From<Point<T>> for Geometry<T> {
    fn from(x: Point<T>) -> Self {
        Self::Point(x)
    }
}
impl<T: CoordNum> From<Line<T>> for Geometry<T> {
    fn from(x: Line<T>) -> Self {
        Self::Line(x)
    }
}
impl<T: CoordNum> From<LineString<T>> for Geometry<T> {
    fn from(x: LineString<T>) -> Self {
        Self::LineString(x)
    }
}
impl<T: CoordNum> From<Polygon<T>> for Geometry<T> {
    fn from(x: Polygon<T>) -> Self {
        Self::Polygon(x)
    }
}
impl<T: CoordNum> From<MultiPoint<T>> for Geometry<T> {
    fn from(x: MultiPoint<T>) -> Self {
        Self::MultiPoint(x)
    }
}
impl<T: CoordNum> From<MultiLineString<T>> for Geometry<T> {
    fn from(x: MultiLineString<T>) -> Self {
        Self::MultiLineString(x)
    }
}
impl<T: CoordNum> From<MultiPolygon<T>> for Geometry<T> {
    fn from(x: MultiPolygon<T>) -> Self {
        Self::MultiPolygon(x)
    }
}

// 在移除已弃用的GeometryCollection::from(single_geom)实现之前禁用
// impl<T: CoordNum> From<GeometryCollection<T>> for Geometry<T> {
//     fn from(x: GeometryCollection<T>) -> Self {
//         Self::GeometryCollection(x)
//     }
// }

impl<T: CoordNum> From<Rect<T>> for Geometry<T> {
    fn from(x: Rect<T>) -> Self {
        Self::Rect(x)
    }
}

impl<T: CoordNum> From<Triangle<T>> for Geometry<T> {
    fn from(x: Triangle<T>) -> Self {
        Self::Triangle(x)
    }
}

impl<T: CoordNum> Geometry<T> {
    /// 如果这个Geometry是一个Point，则返回该Point，否则返回None。
    ///
    /// # 示例
    ///
    /// ```
    /// use geo_types::*;
    /// use std::convert::TryInto;
    ///
    /// let g = Geometry::Point(Point::new(0., 0.));
    /// let p2: Point<f32> = g.try_into().unwrap();
    /// assert_eq!(p2, Point::new(0., 0.,));
    /// ```
    #[deprecated(note = "将在未来版本中移除。请切换到std::convert::TryInto<Point>")]
    pub fn into_point(self) -> Option<Point<T>> {
        if let Geometry::Point(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// 如果这个Geometry是一个LineString，则返回该LineString，否则返回None。
    #[deprecated(note = "将在未来版本中移除。请切换到std::convert::TryInto<LineString>")]
    pub fn into_line_string(self) -> Option<LineString<T>> {
        if let Geometry::LineString(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// 如果这个Geometry是一个Line，则返回该Line，否则返回None。
    #[deprecated(note = "将在未来版本中移除。请切换到std::convert::TryInto<Line>")]
    pub fn into_line(self) -> Option<Line<T>> {
        if let Geometry::Line(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// 如果这个Geometry是一个Polygon，则返回该Polygon，否则返回None。
    #[deprecated(note = "将在未来版本中移除。请切换到std::convert::TryInto<Polygon>")]
    pub fn into_polygon(self) -> Option<Polygon<T>> {
        if let Geometry::Polygon(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// 如果这个Geometry是一个MultiPoint，则返回该MultiPoint，否则返回None。
    #[deprecated(note = "将在未来版本中移除。请切换到std::convert::TryInto<MultiPoint>")]
    pub fn into_multi_point(self) -> Option<MultiPoint<T>> {
        if let Geometry::MultiPoint(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// 如果这个Geometry是一个MultiLineString，则返回该MultiLineString，否则返回None。
    #[deprecated(note = "将在未来版本中移除。请切换到std::convert::TryInto<MultiLineString>")]
    pub fn into_multi_line_string(self) -> Option<MultiLineString<T>> {
        if let Geometry::MultiLineString(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// 如果这个Geometry是一个MultiPolygon，则返回该MultiPolygon，否则返回None。
    #[deprecated(note = "将在未来版本中移除。请切换到std::convert::TryInto<MultiPolygon>")]
    pub fn into_multi_polygon(self) -> Option<MultiPolygon<T>> {
        if let Geometry::MultiPolygon(x) = self {
            Some(x)
        } else {
            None
        }
    }
}

macro_rules! try_from_geometry_impl {
    ($($type: ident),+) => {
        $(
        /// 将Geometry枚举转换为其内部类型。
        ///
        /// 如果枚举情况与您尝试转换的类型不匹配，则会失败。
        impl <T: CoordNum> TryFrom<Geometry<T>> for $type<T> {
            type Error = Error;

            fn try_from(geom: Geometry<T>) -> Result<Self, Self::Error> {
                match geom {
                    Geometry::$type(g) => Ok(g),
                    other => Err(Error::MismatchedGeometry {
                        expected: type_name::<$type<T>>(),
                        found: inner_type_name(other)
                    })
                }
            }
        }
        )+
    }
}

try_from_geometry_impl!(
    Point,
    Line,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    // 在移除已弃用的GeometryCollection::from(single_geom)实现之前禁用
    // GeometryCollection,
    Rect,
    Triangle
);

fn inner_type_name<T>(geometry: Geometry<T>) -> &'static str
where
    T: CoordNum,
{
    match geometry {
        Geometry::Point(_) => type_name::<Point<T>>(),
        Geometry::Line(_) => type_name::<Line<T>>(),
        Geometry::LineString(_) => type_name::<LineString<T>>(),
        Geometry::Polygon(_) => type_name::<Polygon<T>>(),
        Geometry::MultiPoint(_) => type_name::<MultiPoint<T>>(),
        Geometry::MultiLineString(_) => type_name::<MultiLineString<T>>(),
        Geometry::MultiPolygon(_) => type_name::<MultiPolygon<T>>(),
        Geometry::GeometryCollection(_) => type_name::<GeometryCollection<T>>(),
        Geometry::Rect(_) => type_name::<Rect<T>>(),
        Geometry::Triangle(_) => type_name::<Triangle<T>>(),
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for Geometry<T>
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
    /// use geo_types::{Geometry, polygon};
    ///
    /// let a: Geometry<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7., y: 9.), (x: 0., y: 0.)].into();
    /// let b: Geometry<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7.01, y: 9.), (x: 0., y: 0.)].into();
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
        match (self, other) {
            (Geometry::Point(g1), Geometry::Point(g2)) => g1.relative_eq(g2, epsilon, max_relative),
            (Geometry::Line(g1), Geometry::Line(g2)) => g1.relative_eq(g2, epsilon, max_relative),
            (Geometry::LineString(g1), Geometry::LineString(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::Polygon(g1), Geometry::Polygon(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::MultiPoint(g1), Geometry::MultiPoint(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::MultiLineString(g1), Geometry::MultiLineString(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::MultiPolygon(g1), Geometry::MultiPolygon(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::GeometryCollection(g1), Geometry::GeometryCollection(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (Geometry::Rect(g1), Geometry::Rect(g2)) => g1.relative_eq(g2, epsilon, max_relative),
            (Geometry::Triangle(g1), Geometry::Triangle(g2)) => {
                g1.relative_eq(g2, epsilon, max_relative)
            }
            (_, _) => false,
        }
    }
}

#[cfg(any(feature = "approx", test))]
impl<T: AbsDiffEq<Epsilon = T> + CoordNum> AbsDiffEq for Geometry<T> {
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
    /// use geo_types::{Geometry, polygon};
    ///
    /// let a: Geometry<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7., y: 9.), (x: 0., y: 0.)].into();
    /// let b: Geometry<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7.01, y: 9.), (x: 0., y: 0.)].into();
    ///
    /// approx::assert_abs_diff_eq!(a, b, epsilon=0.1);
    /// approx::assert_abs_diff_ne!(a, b, epsilon=0.001);
    /// ```
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        match (self, other) {
            (Geometry::Point(g1), Geometry::Point(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::Line(g1), Geometry::Line(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::LineString(g1), Geometry::LineString(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::Polygon(g1), Geometry::Polygon(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::MultiPoint(g1), Geometry::MultiPoint(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::MultiLineString(g1), Geometry::MultiLineString(g2)) => {
                g1.abs_diff_eq(g2, epsilon)
            }
            (Geometry::MultiPolygon(g1), Geometry::MultiPolygon(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::GeometryCollection(g1), Geometry::GeometryCollection(g2)) => {
                g1.abs_diff_eq(g2, epsilon)
            }
            (Geometry::Rect(g1), Geometry::Rect(g2)) => g1.abs_diff_eq(g2, epsilon),
            (Geometry::Triangle(g1), Geometry::Triangle(g2)) => g1.abs_diff_eq(g2, epsilon),
            (_, _) => false,
        }
    }
}
