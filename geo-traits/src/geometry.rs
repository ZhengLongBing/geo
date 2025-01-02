use std::marker::PhantomData;

#[cfg(feature = "geo-types")]
use geo_types::{
    CoordNum, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};

use crate::{
    Dimensions, GeometryCollectionTrait, LineStringTrait, LineTrait, MultiLineStringTrait,
    MultiPointTrait, MultiPolygonTrait, PointTrait, PolygonTrait, RectTrait, TriangleTrait,
    UnimplementedGeometryCollection, UnimplementedLine, UnimplementedLineString,
    UnimplementedMultiLineString, UnimplementedMultiPoint, UnimplementedMultiPolygon,
    UnimplementedPoint, UnimplementedPolygon, UnimplementedRect, UnimplementedTriangle,
};

/// 用于从通用几何体访问数据的特征。
#[allow(clippy::type_complexity)]
pub trait GeometryTrait {
    /// 此几何体的坐标类型
    type T;

    /// 每个底层点的类型，实现 [PointTrait]
    type PointType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// 每个底层线串的类型，实现 [LineStringTrait]
    type LineStringType<'a>: 'a + LineStringTrait<T = Self::T>
    where
        Self: 'a;

    /// 每个底层多边形的类型，实现 [PolygonTrait]
    type PolygonType<'a>: 'a + PolygonTrait<T = Self::T>
    where
        Self: 'a;

    /// 每个底层多点的类型，实现 [MultiPointTrait]
    type MultiPointType<'a>: 'a + MultiPointTrait<T = Self::T>
    where
        Self: 'a;

    /// 每个底层多线串的类型，实现 [MultiLineStringTrait]
    type MultiLineStringType<'a>: 'a + MultiLineStringTrait<T = Self::T>
    where
        Self: 'a;

    /// 每个底层多多边形的类型，实现 [MultiPolygonTrait]
    type MultiPolygonType<'a>: 'a + MultiPolygonTrait<T = Self::T>
    where
        Self: 'a;

    /// 每个底层几何集合的类型，实现 [GeometryCollectionTrait]
    type GeometryCollectionType<'a>: 'a + GeometryCollectionTrait<T = Self::T>
    where
        Self: 'a;

    /// 每个底层矩形的类型，实现 [RectTrait]
    type RectType<'a>: 'a + RectTrait<T = Self::T>
    where
        Self: 'a;

    /// 每个底层三角形的类型，实现 [TriangleTrait]
    type TriangleType<'a>: 'a + TriangleTrait<T = Self::T>
    where
        Self: 'a;

    /// 每个底层线段的类型，实现 [LineTrait]
    type LineType<'a>: 'a + LineTrait<T = Self::T>
    where
        Self: 'a;

    /// 此几何体的维度
    fn dim(&self) -> Dimensions;

    /// 将此几何体转换为 [`GeometryType`] 枚举，允许向下转型为特定类型
    fn as_type(
        &self,
    ) -> GeometryType<
        '_,
        Self::PointType<'_>,
        Self::LineStringType<'_>,
        Self::PolygonType<'_>,
        Self::MultiPointType<'_>,
        Self::MultiLineStringType<'_>,
        Self::MultiPolygonType<'_>,
        Self::GeometryCollectionType<'_>,
        Self::RectType<'_>,
        Self::TriangleType<'_>,
        Self::LineType<'_>,
    >;
}

/// [GeometryTrait] 中可包含的所有几何类型的枚举。用于从 [GeometryTrait] 中提取具体的几何类型。
#[derive(Debug)]
pub enum GeometryType<'a, P, LS, Y, MP, ML, MY, GC, R, T, L>
where
    P: PointTrait,
    LS: LineStringTrait,
    Y: PolygonTrait,
    MP: MultiPointTrait,
    ML: MultiLineStringTrait,
    MY: MultiPolygonTrait,
    GC: GeometryCollectionTrait,
    R: RectTrait,
    T: TriangleTrait,
    L: LineTrait,
{
    /// 实现 [PointTrait] 的点
    Point(&'a P),
    /// 实现 [LineStringTrait] 的线串
    LineString(&'a LS),
    /// 实现 [PolygonTrait] 的多边形
    Polygon(&'a Y),
    /// 实现 [MultiPointTrait] 的多点
    MultiPoint(&'a MP),
    /// 实现 [MultiLineStringTrait] 的多线串
    MultiLineString(&'a ML),
    /// 实现 [MultiPolygonTrait] 的多多边形
    MultiPolygon(&'a MY),
    /// 实现 [GeometryCollectionTrait] 的几何集合
    GeometryCollection(&'a GC),
    /// 实现 [RectTrait] 的矩形
    Rect(&'a R),
    /// 实现 [TriangleTrait] 的三角形
    Triangle(&'a T),
    /// 实现 [LineTrait] 的线段
    Line(&'a L),
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> GeometryTrait for Geometry<T> {
    type T = T;
    type PointType<'b>
        = Point<Self::T>
    where
        Self: 'b;
    type LineStringType<'b>
        = LineString<Self::T>
    where
        Self: 'b;
    type PolygonType<'b>
        = Polygon<Self::T>
    where
        Self: 'b;
    type MultiPointType<'b>
        = MultiPoint<Self::T>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = MultiLineString<Self::T>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = MultiPolygon<Self::T>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = GeometryCollection<Self::T>
    where
        Self: 'b;
    type RectType<'b>
        = Rect<Self::T>
    where
        Self: 'b;
    type TriangleType<'b>
        = Triangle<Self::T>
    where
        Self: 'b;
    type LineType<'b>
        = Line<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn as_type(
        &self,
    ) -> GeometryType<
        '_,
        Point<T>,
        LineString<T>,
        Polygon<T>,
        MultiPoint<T>,
        MultiLineString<T>,
        MultiPolygon<T>,
        GeometryCollection<T>,
        Rect<T>,
        Triangle<T>,
        Line<T>,
    > {
        match self {
            Geometry::Point(p) => GeometryType::Point(p),
            Geometry::LineString(p) => GeometryType::LineString(p),
            Geometry::Polygon(p) => GeometryType::Polygon(p),
            Geometry::MultiPoint(p) => GeometryType::MultiPoint(p),
            Geometry::MultiLineString(p) => GeometryType::MultiLineString(p),
            Geometry::MultiPolygon(p) => GeometryType::MultiPolygon(p),
            Geometry::GeometryCollection(p) => GeometryType::GeometryCollection(p),
            Geometry::Rect(p) => GeometryType::Rect(p),
            Geometry::Triangle(p) => GeometryType::Triangle(p),
            Geometry::Line(p) => GeometryType::Line(p),
        }
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum + 'a> GeometryTrait for &'a Geometry<T> {
    type T = T;
    type PointType<'b>
        = Point<Self::T>
    where
        Self: 'b;
    type LineStringType<'b>
        = LineString<Self::T>
    where
        Self: 'b;
    type PolygonType<'b>
        = Polygon<Self::T>
    where
        Self: 'b;
    type MultiPointType<'b>
        = MultiPoint<Self::T>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = MultiLineString<Self::T>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = MultiPolygon<Self::T>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = GeometryCollection<Self::T>
    where
        Self: 'b;
    type RectType<'b>
        = Rect<Self::T>
    where
        Self: 'b;
    type TriangleType<'b>
        = Triangle<Self::T>
    where
        Self: 'b;
    type LineType<'b>
        = Line<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn as_type(
        &self,
    ) -> GeometryType<
        '_,
        Point<T>,
        LineString<T>,
        Polygon<T>,
        MultiPoint<T>,
        MultiLineString<T>,
        MultiPolygon<T>,
        GeometryCollection<T>,
        Rect<T>,
        Triangle<T>,
        Line<T>,
    > {
        match self {
            Geometry::Point(p) => GeometryType::Point(p),
            Geometry::LineString(p) => GeometryType::LineString(p),
            Geometry::Polygon(p) => GeometryType::Polygon(p),
            Geometry::MultiPoint(p) => GeometryType::MultiPoint(p),
            Geometry::MultiLineString(p) => GeometryType::MultiLineString(p),
            Geometry::MultiPolygon(p) => GeometryType::MultiPolygon(p),
            Geometry::GeometryCollection(p) => GeometryType::GeometryCollection(p),
            Geometry::Rect(p) => GeometryType::Rect(p),
            Geometry::Triangle(p) => GeometryType::Triangle(p),
            Geometry::Line(p) => GeometryType::Line(p),
        }
    }
}

// 对每个 geo-types 具体类型的专门实现。

macro_rules! impl_specialization {
    ($geometry_type:ident) => {
        #[cfg(feature = "geo-types")]
        impl<T: CoordNum> GeometryTrait for $geometry_type<T> {
            type T = T;
            type PointType<'b>
                = Point<Self::T>
            where
                Self: 'b;
            type LineStringType<'b>
                = LineString<Self::T>
            where
                Self: 'b;
            type PolygonType<'b>
                = Polygon<Self::T>
            where
                Self: 'b;
            type MultiPointType<'b>
                = MultiPoint<Self::T>
            where
                Self: 'b;
            type MultiLineStringType<'b>
                = MultiLineString<Self::T>
            where
                Self: 'b;
            type MultiPolygonType<'b>
                = MultiPolygon<Self::T>
            where
                Self: 'b;
            type GeometryCollectionType<'b>
                = GeometryCollection<Self::T>
            where
                Self: 'b;
            type RectType<'b>
                = Rect<Self::T>
            where
                Self: 'b;
            type TriangleType<'b>
                = Triangle<Self::T>
            where
                Self: 'b;
            type LineType<'b>
                = Line<Self::T>
            where
                Self: 'b;

            fn dim(&self) -> Dimensions {
                Dimensions::Xy
            }

            fn as_type(
                &self,
            ) -> GeometryType<
                '_,
                Point<T>,
                LineString<T>,
                Polygon<T>,
                MultiPoint<T>,
                MultiLineString<T>,
                MultiPolygon<T>,
                GeometryCollection<T>,
                Rect<T>,
                Triangle<T>,
                Line<T>,
            > {
                GeometryType::$geometry_type(self)
            }
        }

        #[cfg(feature = "geo-types")]
        impl<'a, T: CoordNum + 'a> GeometryTrait for &'a $geometry_type<T> {
            type T = T;
            type PointType<'b>
                = Point<Self::T>
            where
                Self: 'b;
            type LineStringType<'b>
                = LineString<Self::T>
            where
                Self: 'b;
            type PolygonType<'b>
                = Polygon<Self::T>
            where
                Self: 'b;
            type MultiPointType<'b>
                = MultiPoint<Self::T>
            where
                Self: 'b;
            type MultiLineStringType<'b>
                = MultiLineString<Self::T>
            where
                Self: 'b;
            type MultiPolygonType<'b>
                = MultiPolygon<Self::T>
            where
                Self: 'b;
            type GeometryCollectionType<'b>
                = GeometryCollection<Self::T>
            where
                Self: 'b;
            type RectType<'b>
                = Rect<Self::T>
            where
                Self: 'b;
            type TriangleType<'b>
                = Triangle<Self::T>
            where
                Self: 'b;
            type LineType<'b>
                = Line<Self::T>
            where
                Self: 'b;

            fn dim(&self) -> Dimensions {
                Dimensions::Xy
            }

            fn as_type(
                &self,
            ) -> GeometryType<
                '_,
                Point<T>,
                LineString<T>,
                Polygon<T>,
                MultiPoint<T>,
                MultiLineString<T>,
                MultiPolygon<T>,
                GeometryCollection<T>,
                Rect<T>,
                Triangle<T>,
                Line<T>,
            > {
                GeometryType::$geometry_type(self)
            }
        }
    };
}

impl_specialization!(Point);
impl_specialization!(LineString);
impl_specialization!(Polygon);
impl_specialization!(MultiPoint);
impl_specialization!(MultiLineString);
impl_specialization!(MultiPolygon);
impl_specialization!(GeometryCollection);
impl_specialization!(Rect);
impl_specialization!(Triangle);
impl_specialization!(Line);

/// An empty struct that implements [GeometryTrait].
///
/// This is used internally for [`UnimplementedGeometryCollection`], so that
/// `UnimplementedGeometryCollection` can be used as the `GeometryCollectionType` of the
/// `GeometryTrait` by implementations that don't have a GeometryCollection concept
pub struct UnimplementedGeometry<T>(PhantomData<T>);

impl<T> GeometryTrait for UnimplementedGeometry<T> {
    type T = T;
    type PointType<'b>
        = UnimplementedPoint<T>
    where
        Self: 'b;
    type LineStringType<'b>
        = UnimplementedLineString<Self::T>
    where
        Self: 'b;
    type PolygonType<'b>
        = UnimplementedPolygon<Self::T>
    where
        Self: 'b;
    type MultiPointType<'b>
        = UnimplementedMultiPoint<Self::T>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = UnimplementedMultiLineString<Self::T>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = UnimplementedMultiPolygon<Self::T>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = UnimplementedGeometryCollection<Self::T>
    where
        Self: 'b;
    type RectType<'b>
        = UnimplementedRect<Self::T>
    where
        Self: 'b;
    type TriangleType<'b>
        = UnimplementedTriangle<Self::T>
    where
        Self: 'b;
    type LineType<'b>
        = UnimplementedLine<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn as_type(
        &self,
    ) -> GeometryType<
        '_,
        Self::PointType<'_>,
        Self::LineStringType<'_>,
        Self::PolygonType<'_>,
        Self::MultiPointType<'_>,
        Self::MultiLineStringType<'_>,
        Self::MultiPolygonType<'_>,
        Self::GeometryCollectionType<'_>,
        Self::RectType<'_>,
        Self::TriangleType<'_>,
        Self::LineType<'_>,
    > {
        unimplemented!()
    }
}
