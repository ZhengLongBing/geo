use crate::{
    CoordNum, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};
use std::borrow::Cow;

/// `GeometryCow` 是一个类似于 [`Geometry`] 的 "one of" 枚举，
/// 但 `GeometryCow` 的内部类型可以是一个引用，而不是拥有的。
///
/// 这是一种在不 `移动` 的情况下将内部类型“升级”为类似 `Geometry` 的方式。
///
/// 例如，请参见使用 `GeometryCow` 的 [`Relate`] 特性。
#[derive(PartialEq, Debug, Hash, Clone)]
pub(crate) enum GeometryCow<'a, T>
where
    T: CoordNum,
{
    Point(Cow<'a, Point<T>>),
    Line(Cow<'a, Line<T>>),
    LineString(Cow<'a, LineString<T>>),
    Polygon(Cow<'a, Polygon<T>>),
    MultiPoint(Cow<'a, MultiPoint<T>>),
    MultiLineString(Cow<'a, MultiLineString<T>>),
    MultiPolygon(Cow<'a, MultiPolygon<T>>),
    GeometryCollection(Cow<'a, GeometryCollection<T>>),
    Rect(Cow<'a, Rect<T>>),
    Triangle(Cow<'a, Triangle<T>>),
}

impl<'a, T: CoordNum> From<&'a Geometry<T>> for GeometryCow<'a, T> {
    fn from(geometry: &'a Geometry<T>) -> Self {
        // 将 Geometry 类型转换为 GeometryCow，其内部元素为引用
        match geometry {
            Geometry::Point(g) => GeometryCow::Point(Cow::Borrowed(g)),
            Geometry::Line(g) => GeometryCow::Line(Cow::Borrowed(g)),
            Geometry::LineString(g) => GeometryCow::LineString(Cow::Borrowed(g)),
            Geometry::Polygon(g) => GeometryCow::Polygon(Cow::Borrowed(g)),
            Geometry::MultiPoint(g) => GeometryCow::MultiPoint(Cow::Borrowed(g)),
            Geometry::MultiLineString(g) => GeometryCow::MultiLineString(Cow::Borrowed(g)),
            Geometry::MultiPolygon(g) => GeometryCow::MultiPolygon(Cow::Borrowed(g)),
            Geometry::GeometryCollection(g) => GeometryCow::GeometryCollection(Cow::Borrowed(g)),
            Geometry::Rect(g) => GeometryCow::Rect(Cow::Borrowed(g)),
            Geometry::Triangle(g) => GeometryCow::Triangle(Cow::Borrowed(g)),
        }
    }
}

impl<'a, T: CoordNum> From<&'a Point<T>> for GeometryCow<'a, T> {
    fn from(point: &'a Point<T>) -> Self {
        // 将 Point 转换为 GeometryCow，其内部元素为引用
        GeometryCow::Point(Cow::Borrowed(point))
    }
}

impl<'a, T: CoordNum> From<&'a LineString<T>> for GeometryCow<'a, T> {
    fn from(line_string: &'a LineString<T>) -> Self {
        // 将 LineString 转换为 GeometryCow，其内部元素为引用
        GeometryCow::LineString(Cow::Borrowed(line_string))
    }
}

impl<'a, T: CoordNum> From<&'a Line<T>> for GeometryCow<'a, T> {
    fn from(line: &'a Line<T>) -> Self {
        // 将 Line 转换为 GeometryCow，其内部元素为引用
        GeometryCow::Line(Cow::Borrowed(line))
    }
}

impl<'a, T: CoordNum> From<&'a Polygon<T>> for GeometryCow<'a, T> {
    fn from(polygon: &'a Polygon<T>) -> Self {
        // 将 Polygon 转换为 GeometryCow，其内部元素为引用
        GeometryCow::Polygon(Cow::Borrowed(polygon))
    }
}

impl<'a, T: CoordNum> From<&'a MultiPoint<T>> for GeometryCow<'a, T> {
    fn from(multi_point: &'a MultiPoint<T>) -> GeometryCow<'a, T> {
        // 将 MultiPoint 转换为 GeometryCow，其内部元素为引用
        GeometryCow::MultiPoint(Cow::Borrowed(multi_point))
    }
}

impl<'a, T: CoordNum> From<&'a MultiLineString<T>> for GeometryCow<'a, T> {
    fn from(multi_line_string: &'a MultiLineString<T>) -> Self {
        // 将 MultiLineString 转换为 GeometryCow，其内部元素为引用
        GeometryCow::MultiLineString(Cow::Borrowed(multi_line_string))
    }
}

impl<'a, T: CoordNum> From<&'a MultiPolygon<T>> for GeometryCow<'a, T> {
    fn from(multi_polygon: &'a MultiPolygon<T>) -> Self {
        // 将 MultiPolygon 转换为 GeometryCow，其内部元素为引用
        GeometryCow::MultiPolygon(Cow::Borrowed(multi_polygon))
    }
}

impl<'a, T: CoordNum> From<&'a GeometryCollection<T>> for GeometryCow<'a, T> {
    fn from(geometry_collection: &'a GeometryCollection<T>) -> Self {
        // 将 GeometryCollection 转换为 GeometryCow，其内部元素为引用
        GeometryCow::GeometryCollection(Cow::Borrowed(geometry_collection))
    }
}

impl<'a, T: CoordNum> From<&'a Rect<T>> for GeometryCow<'a, T> {
    fn from(rect: &'a Rect<T>) -> Self {
        // 将 Rect 转换为 GeometryCow，其内部元素为引用
        GeometryCow::Rect(Cow::Borrowed(rect))
    }
}

impl<'a, T: CoordNum> From<&'a Triangle<T>> for GeometryCow<'a, T> {
    fn from(triangle: &'a Triangle<T>) -> Self {
        // 将 Triangle 转换为 GeometryCow，其内部元素为引用
        GeometryCow::Triangle(Cow::Borrowed(triangle))
    }
}

impl<T: CoordNum> From<Point<T>> for GeometryCow<'_, T> {
    fn from(point: Point<T>) -> Self {
        // 将拥有的 Point 转换为 GeometryCow，其内部元素为拥有的
        GeometryCow::Point(Cow::Owned(point))
    }
}

impl<T: CoordNum> From<LineString<T>> for GeometryCow<'_, T> {
    fn from(line_string: LineString<T>) -> Self {
        // 将拥有的 LineString 转换为 GeometryCow，其内部元素为拥有的
        GeometryCow::LineString(Cow::Owned(line_string))
    }
}

impl<T: CoordNum> From<Line<T>> for GeometryCow<'_, T> {
    fn from(line: Line<T>) -> Self {
        // 将拥有的 Line 转换为 GeometryCow，其内部元素为拥有的
        GeometryCow::Line(Cow::Owned(line))
    }
}

impl<T: CoordNum> From<Polygon<T>> for GeometryCow<'_, T> {
    fn from(polygon: Polygon<T>) -> Self {
        // 将拥有的 Polygon 转换为 GeometryCow，其内部元素为拥有的
        GeometryCow::Polygon(Cow::Owned(polygon))
    }
}

impl<'a, T: CoordNum> From<MultiPoint<T>> for GeometryCow<'a, T> {
    fn from(multi_point: MultiPoint<T>) -> GeometryCow<'a, T> {
        // 将拥有的 MultiPoint 转换为 GeometryCow，其内部元素为拥有的
        GeometryCow::MultiPoint(Cow::Owned(multi_point))
    }
}

impl<T: CoordNum> From<MultiLineString<T>> for GeometryCow<'_, T> {
    fn from(multi_line_string: MultiLineString<T>) -> Self {
        // 将拥有的 MultiLineString 转换为 GeometryCow，其内部元素为拥有的
        GeometryCow::MultiLineString(Cow::Owned(multi_line_string))
    }
}

impl<T: CoordNum> From<MultiPolygon<T>> for GeometryCow<'_, T> {
    fn from(multi_polygon: MultiPolygon<T>) -> Self {
        // 将拥有的 MultiPolygon 转换为 GeometryCow，其内部元素为拥有的
        GeometryCow::MultiPolygon(Cow::Owned(multi_polygon))
    }
}

impl<T: CoordNum> From<GeometryCollection<T>> for GeometryCow<'_, T> {
    fn from(geometry_collection: GeometryCollection<T>) -> Self {
        // 将拥有的 GeometryCollection 转换为 GeometryCow，其内部元素为拥有的
        GeometryCow::GeometryCollection(Cow::Owned(geometry_collection))
    }
}

impl<T: CoordNum> From<Rect<T>> for GeometryCow<'_, T> {
    fn from(rect: Rect<T>) -> Self {
        // 将拥有的 Rect 转换为 GeometryCow，其内部元素为拥有的
        GeometryCow::Rect(Cow::Owned(rect))
    }
}

impl<T: CoordNum> From<Triangle<T>> for GeometryCow<'_, T> {
    fn from(triangle: Triangle<T>) -> Self {
        // 将拥有的 Triangle 转换为 GeometryCow，其内部元素为拥有的
        GeometryCow::Triangle(Cow::Owned(triangle))
    }
}

impl<T: CoordNum> From<Geometry<T>> for GeometryCow<'_, T> {
    fn from(geometry: Geometry<T>) -> Self {
        // 将拥有的 Geometry 类型转换为 GeometryCow，其内部元素为拥有的
        match geometry {
            Geometry::Point(point) => GeometryCow::from(point),
            Geometry::Line(line) => GeometryCow::from(line),
            Geometry::LineString(line_string) => GeometryCow::from(line_string),
            Geometry::Polygon(polygon) => GeometryCow::from(polygon),
            Geometry::MultiPoint(multi_point) => GeometryCow::from(multi_point),
            Geometry::MultiLineString(multi_line_string) => GeometryCow::from(multi_line_string),
            Geometry::MultiPolygon(multi_polygon) => GeometryCow::from(multi_polygon),
            Geometry::GeometryCollection(geometry_collection) => {
                GeometryCow::from(geometry_collection)
            }
            Geometry::Rect(rect) => GeometryCow::from(rect),
            Geometry::Triangle(triangle) => GeometryCow::from(triangle),
        }
    }
}
